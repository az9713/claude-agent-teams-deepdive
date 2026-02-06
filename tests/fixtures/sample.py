# A sample Python file for testing TODO detection

def main():
    # TODO: Add type hints
    x = 42

    # FIXME(alice, p:high): This needs better error handling
    try:
        y = x / 0
    except:
        pass

    # BUG: Division by zero not properly handled
    print(x)

if __name__ == "__main__":
    main()
