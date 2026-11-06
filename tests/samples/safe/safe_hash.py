import hashlib
import os

def hash_password(password: str) -> bytes:
    salt = os.urandom(16)
    return salt + hashlib.scrypt(password.encode(), salt=salt, n=2**14, r=8, p=1)
