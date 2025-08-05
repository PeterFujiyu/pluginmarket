#!/usr/bin/env python3
"""
Simple HTTP server with API proxy to work around CORS issues
"""
import http.server
import socketserver
import urllib.request
import urllib.parse
import json
import os

PORT = 8080
BACKEND_URL = os.getenv('BACKEND_URL', 'http://localhost:3000')

class ProxyHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        # Override to reduce verbose logging, only log errors
        if 'error' in format.lower() or 'exception' in format.lower():
            super().log_message(format, *args)
    def do_GET(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            super().do_GET()
    
    def do_POST(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            self.send_error(405, "Method not allowed")
    
    def do_PUT(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            self.send_error(405, "Method not allowed")
    
    def do_DELETE(self):
        if self.path.startswith('/api/'):
            self.proxy_request()
        else:
            self.send_error(405, "Method not allowed")
    
    def do_OPTIONS(self):
        if self.path.startswith('/api/'):
            self.send_response(200)
            self.send_header('Access-Control-Allow-Origin', '*')
            self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
            self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
            self.end_headers()
        else:
            super().do_OPTIONS()
    
    def proxy_request(self):
        try:
            # Build the backend URL
            backend_url = BACKEND_URL + self.path
            
            # Prepare the request
            headers = {}
            for header, value in self.headers.items():
                if header.lower() not in ['host', 'connection']:
                    headers[header] = value
            
            # Get request body if it's a POST request
            content_length = int(self.headers.get('Content-Length', 0))
            post_data = None
            if content_length > 0:
                post_data = self.rfile.read(content_length)
            
            # Make the request to backend
            req = urllib.request.Request(backend_url, data=post_data, headers=headers, method=self.command)
            
            with urllib.request.urlopen(req) as response:
                # Send response
                self.send_response(response.status)
                
                # Copy headers
                for header, value in response.headers.items():
                    if header.lower() not in ['connection', 'transfer-encoding']:
                        self.send_header(header, value)
                
                # Add CORS headers
                self.send_header('Access-Control-Allow-Origin', '*')
                self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
                self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
                
                self.end_headers()
                
                # Copy response body
                self.wfile.write(response.read())
                
        except BrokenPipeError:
            # Client disconnected, silently ignore
            print(f"Client disconnected during request to {self.path}")
        except ConnectionResetError:
            # Connection reset by client, silently ignore
            print(f"Connection reset during request to {self.path}")
        except Exception as e:
            print(f"Proxy error: {e}")
            try:
                self.send_error(500, f"Proxy error: {str(e)}")
            except (BrokenPipeError, ConnectionResetError):
                # Client already disconnected, can't send error response
                pass

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    # Custom TCPServer that handles errors gracefully
    class RobustTCPServer(socketserver.TCPServer):
        def handle_error(self, request, client_address):
            # Suppress BrokenPipeError and ConnectionResetError
            import sys
            exc_type, exc_value, exc_traceback = sys.exc_info()
            if exc_type in (BrokenPipeError, ConnectionResetError):
                print(f"Client {client_address} disconnected")
                return
            # Log other errors normally
            super().handle_error(request, client_address)
    
    with RobustTCPServer(("", PORT), ProxyHandler) as httpd:
        httpd.allow_reuse_address = True
        print(f"Serving at http://localhost:{PORT}")
        print(f"Proxying /api/* requests to {BACKEND_URL}")
        print("Press Ctrl+C to stop")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down proxy server...")
            httpd.shutdown()
