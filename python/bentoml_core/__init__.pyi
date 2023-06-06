def validate_tag_str(tag_str: str) -> None:
    """
    Validates the tag string is in the correct format.

    Raises:
        InvalidTagException: If the tag string is not in the correct format.
    """
    pass

class Tag:
    def __init__(self, name: str, version: str | None = None) -> None:
        pass
    def __str__(self) -> str:
        pass
    def __repr__(self) -> str:
        pass
    def __eq__(self, other: Tag) -> bool:
        pass
    def __lt__(self, other: Tag) -> bool:
        pass
    def __hash__(self) -> int:
        pass
    @staticmethod
    def from_taglike(taglike: str) -> Tag:
        pass
    @staticmethod
    def from_str(tag_str: str) -> Tag:
        pass
