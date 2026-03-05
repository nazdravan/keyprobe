"""Shared by the asset generators: reads assets/theme.toml (Python 3.11+, stdlib only)."""
import pathlib
import tomllib

ASSETS = pathlib.Path(__file__).resolve().parent
ROOT = ASSETS.parent


def load():
    with open(ASSETS / "theme.toml", "rb") as f:
        return tomllib.load(f)


def rgba(hex_colour, alpha=255):
    h = hex_colour.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), alpha)
