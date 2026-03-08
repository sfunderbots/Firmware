#!/usr/bin/env python3
import argparse
import socket
import struct
import threading
from collections import deque
from typing import Deque, Dict, List

from flask import Flask, Response, jsonify, request

IMU_MAGIC = 0x494D5530
IMU_STRUCT = struct.Struct("!IBBHIQQhhhhhhhiiiiiii")


class SampleStore:
    def __init__(self, maxlen: int) -> None:
        self._samples: Deque[Dict] = deque(maxlen=maxlen)
        self._lock = threading.Lock()

    def add(self, sample: Dict) -> None:
        with self._lock:
            self._samples.append(sample)

    def latest(self, count: int) -> List[Dict]:
        with self._lock:
            if count <= 0:
                return []
            return list(self._samples)[-count:]

def parse_payload(payload: bytes) -> Dict:
    if len(payload) < IMU_STRUCT.size:
        raise ValueError("payload too short")
    unpacked = IMU_STRUCT.unpack(payload[: IMU_STRUCT.size])
    (
        magic,
        version,
        _reserved,
        payload_len,
        seq,
        mono_ns,
        real_ns,
        temp_raw,
        gx_raw,
        gy_raw,
        gz_raw,
        ax_raw,
        ay_raw,
        az_raw,
        temp_mdegc,
        gx_mdps,
        gy_mdps,
        gz_mdps,
        ax_ums2,
        ay_ums2,
        az_ums2,
    ) = unpacked

    if magic != IMU_MAGIC:
        raise ValueError("bad magic")
    if payload_len != IMU_STRUCT.size:
        raise ValueError("payload length mismatch")

    return {
        "version": version,
        "seq": seq,
        "mono_ns": mono_ns,
        "real_ns": real_ns,
        "real_s": real_ns / 1_000_000_000.0,
        "temp_raw": temp_raw,
        "gyro_raw": [gx_raw, gy_raw, gz_raw],
        "accel_raw": [ax_raw, ay_raw, az_raw],
        "temp_c": temp_mdegc / 1000.0,
        "gyro_dps": [gx_mdps / 1000.0, gy_mdps / 1000.0, gz_mdps / 1000.0],
        "accel_ms2": [ax_ums2 / 1_000_000.0, ay_ums2 / 1_000_000.0, az_ums2 / 1_000_000.0],
    }


def capture_loop(listen_host: str, udp_port: int, store: SampleStore, stop_evt: threading.Event) -> None:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((listen_host, udp_port))
    sock.settimeout(1.0)

    while not stop_evt.is_set():
        try:
            payload, _ = sock.recvfrom(2048)
        except socket.timeout:
            continue
        except OSError:
            break

        try:
            sample = parse_payload(payload)
            store.add(sample)
        except ValueError:
            continue

    sock.close()


def create_app(store: SampleStore) -> Flask:
    app = Flask(__name__)

    @app.route("/")
    def index() -> Response:
        return Response(
            """<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <title>IMU Ethernet Viewer</title>
  <style>
    body { font-family: sans-serif; margin: 18px; background: #f4f6f8; color: #111; }
    h1 { margin: 0 0 10px; }
    .row { display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 10px; }
    .card { background: #fff; border: 1px solid #d5dce3; border-radius: 8px; padding: 10px; }
    canvas { background: #fff; border: 1px solid #d5dce3; border-radius: 8px; display: block; margin-bottom: 10px; }
    #status { font-size: 14px; }
  </style>
</head>
<body>
  <h1>IMU Ethernet Viewer</h1>
  <div id="status" class="row"></div>
  <canvas id="gyro" width="1100" height="240"></canvas>
  <canvas id="accel" width="1100" height="240"></canvas>
  <canvas id="temp" width="1100" height="180"></canvas>
  <script>
    const statusEl = document.getElementById("status");
    const gyroCanvas = document.getElementById("gyro");
    const accelCanvas = document.getElementById("accel");
    const tempCanvas = document.getElementById("temp");
    let lastSeq = -1;

    function drawAxes(ctx, w, h, title) {
      ctx.clearRect(0, 0, w, h);
      ctx.fillStyle = "#111";
      ctx.font = "14px sans-serif";
      ctx.fillText(title, 10, 18);
      ctx.strokeStyle = "#c7d1db";
      ctx.beginPath();
      ctx.moveTo(40, 30); ctx.lineTo(40, h - 20);
      ctx.lineTo(w - 10, h - 20);
      ctx.stroke();
    }

    function drawSeries(canvas, title, ys, labels, colors) {
      const ctx = canvas.getContext("2d");
      const w = canvas.width;
      const h = canvas.height;
      drawAxes(ctx, w, h, title);
      if (ys.length === 0 || ys[0].length < 2) return;

      let ymin = Infinity, ymax = -Infinity;
      for (const s of ys) {
        for (const v of s) {
          if (v < ymin) ymin = v;
          if (v > ymax) ymax = v;
        }
      }
      if (ymin === ymax) { ymin -= 1; ymax += 1; }
      const n = ys[0].length;
      const x0 = 40, y0 = h - 20, pw = w - 50, ph = h - 50;
      const xAt = i => x0 + (i * pw) / (n - 1);
      const yAt = v => y0 - ((v - ymin) * ph) / (ymax - ymin);

      ctx.fillStyle = "#333";
      ctx.font = "12px sans-serif";
      ctx.fillText(ymax.toFixed(2), 4, 36);
      ctx.fillText(ymin.toFixed(2), 4, h - 24);

      for (let k = 0; k < ys.length; k++) {
        ctx.strokeStyle = colors[k];
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        const s = ys[k];
        ctx.moveTo(xAt(0), yAt(s[0]));
        for (let i = 1; i < s.length; i++) ctx.lineTo(xAt(i), yAt(s[i]));
        ctx.stroke();
      }

      let lx = w - 220;
      for (let k = 0; k < labels.length; k++) {
        ctx.fillStyle = colors[k];
        ctx.fillRect(lx, 10, 10, 10);
        ctx.fillStyle = "#111";
        ctx.fillText(labels[k], lx + 14, 19);
        lx += 68;
      }
    }

    async function refresh() {
      const r = await fetch("/samples?count=400");
      const data = await r.json();
      const samples = data.samples || [];
      if (samples.length === 0) {
        statusEl.textContent = "No samples yet...";
        return;
      }

      const s = samples[samples.length - 1];
      const dt = new Date(s.real_s * 1000).toISOString();
      statusEl.textContent = `seq=${s.seq}  temp=${s.temp_c.toFixed(2)} C  last=${dt}`;
      lastSeq = s.seq;

      const gx = samples.map(v => v.gyro_dps[0]);
      const gy = samples.map(v => v.gyro_dps[1]);
      const gz = samples.map(v => v.gyro_dps[2]);
      drawSeries(gyroCanvas, "Gyro (dps)", [gx, gy, gz], ["gx", "gy", "gz"], ["#d04a3a", "#2f7ed8", "#3b9c5d"]);

      const ax = samples.map(v => v.accel_ms2[0]);
      const ay = samples.map(v => v.accel_ms2[1]);
      const az = samples.map(v => v.accel_ms2[2]);
      drawSeries(accelCanvas, "Accel (m/s^2)", [ax, ay, az], ["ax", "ay", "az"], ["#d04a3a", "#2f7ed8", "#3b9c5d"]);

      const tc = samples.map(v => v.temp_c);
      drawSeries(tempCanvas, "Temperature (C)", [tc], ["temp"], ["#8e44ad"]);
    }

    setInterval(() => refresh().catch(console.error), 200);
    refresh().catch(console.error);
  </script>
</body>
</html>""",
            mimetype="text/html",
        )

    @app.route("/samples")
    def samples() -> Response:
        count = request.args.get("count", default=300, type=int)
        count = max(1, min(count, 5000))
        return jsonify({"samples": store.latest(count)})

    return app


def main() -> None:
    parser = argparse.ArgumentParser(description="Receive IMU UDP datagrams and serve live plots.")
    parser.add_argument("--listen-host", default="0.0.0.0", help="UDP bind host (default: 0.0.0.0)")
    parser.add_argument("--udp-port", type=int, default=5005, help="UDP bind port (default: 5005)")
    parser.add_argument("--host", default="0.0.0.0", help="Web bind host (default: 0.0.0.0)")
    parser.add_argument("--port", type=int, default=5000, help="Web bind port (default: 5000)")
    parser.add_argument("--buffer", type=int, default=5000, help="Max stored samples (default: 5000)")
    args = parser.parse_args()

    store = SampleStore(maxlen=args.buffer)
    stop_evt = threading.Event()
    t = threading.Thread(
        target=capture_loop, args=(args.listen_host, args.udp_port, store, stop_evt), daemon=True
    )
    t.start()

    app = create_app(store)
    try:
        app.run(host=args.host, port=args.port, threaded=True)
    finally:
        stop_evt.set()
        t.join(timeout=1.0)


if __name__ == "__main__":
    main()
