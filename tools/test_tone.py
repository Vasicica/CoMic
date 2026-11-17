"""Trimite un ton de test catre receptorul CoMic, fara telefon si fara Android.

Foloseste doar biblioteca standard. Scop: sa verifici intregul lant de pe
Windows — receptor, VB-Cable, microfonul vazut de CS2 sau Discord — inainte sa
existe aplicatia de telefon. Daca auzi tonul in jocul tau, partea de calculator
este gata.

    python tools/test_tone.py                 # ton de 440 Hz, 10 secunde
    python tools/test_tone.py --seconds 30 --loss 5
"""

import argparse
import math
import random
import socket
import struct
import sys
import time

MAGIC = 0x434D
VERSION = 1
CODEC_PCM_S16LE = 0
SAMPLE_RATE = 48_000
FRAME_MS = 10
FRAME_SAMPLES = SAMPLE_RATE // 1000 * FRAME_MS  # 480
HEADER = struct.Struct(">HBBII")


def build_frame(sequence: int, samples: list[int]) -> bytes:
    """Un pachet CoMic: header de 12 bytes plus PCM semnat pe 16 biti."""
    header = HEADER.pack(
        MAGIC,
        VERSION,
        CODEC_PCM_S16LE,
        sequence & 0xFFFFFFFF,
        (sequence * FRAME_SAMPLES) & 0xFFFFFFFF,
    )
    return header + struct.pack(f"<{len(samples)}h", *samples)


def tone_samples(sequence: int, frequency: float, amplitude: float) -> list[int]:
    """Un cadru de 10 ms de sinusoida, continuu peste cadre."""
    start = sequence * FRAME_SAMPLES
    peak = int(amplitude * 32_767)
    return [
        int(peak * math.sin(2 * math.pi * frequency * (start + index) / SAMPLE_RATE))
        for index in range(FRAME_SAMPLES)
    ]


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generator de ton de test CoMic")
    parser.add_argument("--host", default="127.0.0.1", help="adresa receptorului")
    parser.add_argument("--port", type=int, default=45_678, help="portul receptorului")
    parser.add_argument("--frequency", type=float, default=440.0, help="frecventa in Hz")
    parser.add_argument("--amplitude", type=float, default=0.3, help="amplitudine 0..1")
    parser.add_argument("--seconds", type=float, default=10.0, help="durata")
    parser.add_argument(
        "--loss",
        type=float,
        default=0.0,
        help="procent de pachete aruncate intentionat, ca sa testezi buffer-ul",
    )
    parser.add_argument(
        "--reorder",
        action="store_true",
        help="trimite unele cadre in alta ordine, ca sa testezi reordonarea",
    )
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    if not 0.0 <= arguments.amplitude <= 1.0:
        print("amplitudinea trebuie sa fie intre 0 si 1", file=sys.stderr)
        return 2

    destination = (arguments.host, arguments.port)
    sender = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    total_frames = int(arguments.seconds * 1000 / FRAME_MS)

    print(f"trimit {arguments.frequency:g} Hz catre {arguments.host}:{arguments.port}")
    print(f"{total_frames} cadre, {arguments.seconds:g} s. Ctrl+C pentru oprire.")

    started = time.perf_counter()
    held_back: bytes | None = None
    sent = 0

    try:
        for sequence in range(total_frames):
            packet = build_frame(sequence, tone_samples(sequence, arguments.frequency, arguments.amplitude))

            if random.uniform(0, 100) < arguments.loss:
                pass  # pachet pierdut intentionat
            elif arguments.reorder and held_back is None and sequence % 50 == 7:
                held_back = packet  # il trimitem dupa urmatorul
            else:
                sender.sendto(packet, destination)
                sent += 1
                if held_back is not None:
                    sender.sendto(held_back, destination)
                    sent += 1
                    held_back = None

            # Ritm constant: ne raportam la ceasul de start, nu adunam intarzieri.
            target = started + (sequence + 1) * FRAME_MS / 1000
            remaining = target - time.perf_counter()
            if remaining > 0:
                time.sleep(remaining)
    except KeyboardInterrupt:
        print("\noprit")
    finally:
        sender.close()

    print(f"trimise {sent} pachete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
