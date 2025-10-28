import sys

def main():
    sys.stdout.write("Status: 500 Internal Server Error\r\n")
    sys.stdout.write("Content-Type: text/plain\r\n")
    sys.stdout.write("\r\n")
    sys.stdout.write("Intentional 500 error sent!\n")

if __name__ == "__main__":
    main()