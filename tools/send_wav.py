"""Trimite un fisier WAV catre receptorul CoMic, ca si cum ar veni de pe telefon.

Foloseste doar biblioteca standard. Scop: sa poti asculta vocile din catalog cu
vorbire adevarata, inainte sa existe aplicatia de Android.

    python tools/send_wav.py vorbire.wav
    python tools/send_wav.py vorbire.wav --host 192.168.1.5
"""

import argparse
import socket
import struct
import sys
import time
import wave

MAGIC = 0x434D
VERSION = 1
CODEC_PCM_S16LE = 0
SAMPLE_RATE = 48_000
FRAME_MS = 10
FRAME_SAMPLES = SAMPLE_RATE // 1000 * FRAME_MS
HEADER = struct.Struct(">HBBII")


def read_wav_as_mono_48k(path: str) -> list[int]:
    """Citeste un WAV si il aduce la 48 kHz mono, pe 16 biti."""
    with wave.open(path, "rb") as source:
        if source.getsampwidth() != 2:
            raise ValueError("doar WAV pe 16 biti este suportat")

        channels = source.getnchannels()
        rate = source.getframerate()
        raw = source.readframes(source.getnframes())

    samples = list(struct.unpack(f"<{len(raw) // 2}h", raw))

    if channels > 1:
        # Media canalelor, nu doar primul: altfel pierdem jumatate din semnal.
        samples = [
            sum(samples[index : index + channels]) // channels
            for index in range(0, len(samples) - channels + 1, channels)
        ]

    if rate == SAMPLE_RATE:
        return samples

    # Reesantionare liniara. Suficienta pentru o demonstratie; in program
    # rata este fixa, deci nu avem nevoie de ceva mai bun aici.
    ratio = SAMPLE_RATE / rate
    resampled_length = int(len(samples) * ratio)
    resampled = []
    for index in range(resampled_length):
        position = index / ratio
        lower = int(position)
        upper = min(lower + 1, len(samples) - 1)
        fraction = position - lower
        resampled.append(int(samples[lower] * (1 - fraction) + samples[upper] * fraction))
    return resampled


def build_frame(sequence: int, samples: list[int]) -> bytes:
    header = HEADER.pack(
        MAGIC,
        VERSION,
        CODEC_PCM_S16LE,
        sequence & 0xFFFFFFFF,
        (sequence * FRAME_SAMPLES) & 0xFFFFFFFF,
    )
    return header + struct.pack(f"<{len(samples)}h", *samples)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Trimite un WAV catre receptorul CoMic")
    parser.add_argument("file", help="fisierul WAV de trimis")
    parser.add_argument("--host", default="127.0.0.1", help="adresa receptorului")
    parser.add_argument("--port", type=int, default=45_678, help="portul receptorului")
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()

    try:
        samples = read_wav_as_mono_48k(arguments.file)
    except (OSError, wave.Error, ValueError) as error:
        print(f"nu pot citi {arguments.file}: {error}", file=sys.stderr)
        return 1

    destination = (arguments.host, arguments.port)
    sender = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    total_frames = len(samples) // FRAME_SAMPLES

    print(f"trimit {total_frames * FRAME_MS / 1000:.1f} s catre {arguments.host}:{arguments.port}")

    started = time.perf_counter()
    try:
        for sequence in range(total_frames):
            start = sequence * FRAME_SAMPLES
            frame = samples[start : start + FRAME_SAMPLES]
            sender.sendto(build_frame(sequence, frame), destination)

            target = started + (sequence + 1) * FRAME_MS / 1000
            remaining = target - time.perf_counter()
            if remaining > 0:
                time.sleep(remaining)
    except KeyboardInterrupt:
        print("\noprit")
    finally:
        sender.close()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
