'''Common utilities'''
import socketserver
import string
import secrets

def get_free_port():
    """Return free to use TCP port"""
    with socketserver.TCPServer(("localhost", 0), None) as s:
        free_port = s.server_address[1]
    return free_port

def generate_password(pass_len: int = 10):
    '''Password generator'''
    alphabet = string.ascii_letters + string.digits
    password = ''.join(secrets.choice(alphabet) for i in range(pass_len))
    return password