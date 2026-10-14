#!/usr/bin/env python3
"""HTTP handler for human observation notes.

Runs as a tiny HTTP server behind Caddy. Receives human observations
from chipmunk-talk.js and writes them to squirrel's journal.

Usage: python3 note_handler.py
Listens on 127.0.0.1:9340
"""

import json
import sys
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path

JOURNAL_DIR = Path("/opt/membrane/squirrel-journal/human-notes")
PORT = 9340


class NoteHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass  # quiet

    def do_POST(self):
        if self.path != "/chipmunk/note":
            self.send_response(404)
            self.end_headers()
            return

        length = int(self.headers.get("Content-Length", 0))
        if length > 4096:
            self.send_response(413)
            self.end_headers()
            return

        body = self.rfile.read(length)
        try:
            data = json.loads(body)
        except (json.JSONDecodeError, UnicodeDecodeError):
            self.send_response(400)
            self.end_headers()
            return

        note = data.get("note", "").strip()
        session = data.get("session", "unknown")
        context = data.get("context", {})

        if not note or len(note) > 1000:
            self.send_response(400)
            self.end_headers()
            return

        now = datetime.now(timezone.utc)
        entry = {
            "ts": now.isoformat(),
            "type": "human_observation",
            "session": session,
            "note": note,
            "context": context,
        }

        JOURNAL_DIR.mkdir(parents=True, exist_ok=True)
        today = now.strftime("%Y-%m-%d")
        journal_file = JOURNAL_DIR / f"{today}.jsonl"
        with open(journal_file, "a") as f:
            f.write(json.dumps(entry) + "\n")

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps({"ok": True}).encode())

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()


def main():
    server = HTTPServer(("127.0.0.1", PORT), NoteHandler)
    print(f"note handler listening on 127.0.0.1:{PORT}", file=sys.stderr)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
