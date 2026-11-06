from pathlib import Path

BASE = Path("/srv/files").resolve()

def resolve(name: str) -> Path:
    target = (BASE / name).resolve()
    if BASE not in target.parents:
        raise ValueError("invalid path")
    return target
