class BentoMLException(Exception):
    error_code: str
    message: str