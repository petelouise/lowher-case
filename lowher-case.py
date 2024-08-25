import re

# Compile regex patterns
CODE_BLOCK_PATTERN = re.compile(r"(```[\s\S]*?```|`[^`\n]+`)")
CAPITALIZED_WORD_PATTERN = re.compile(r"\b[A-Z][a-z]+\b")
ACRONYM_PATTERN = re.compile(r"\b[A-Z]{2,}\b")

def mark_special_cases(text):
    # Find all special cases (code blocks, capitalized words, acronyms)
    special_cases = (CODE_BLOCK_PATTERN.findall(text) +
                     CAPITALIZED_WORD_PATTERN.findall(text) +
                     ACRONYM_PATTERN.findall(text))
    
    # Sort special cases by their position in the text (to handle overlaps)
    special_cases.sort(key=lambda x: text.index(x))
    
    # Replace special cases with placeholders
    placeholders = []
    for i, case in enumerate(special_cases):
        placeholder = f"__SPECIAL_CASE_{i}__"
        text = text.replace(case, placeholder, 1)
        placeholders.append((placeholder, case))
    
    return text, placeholders

def unmark_special_cases(text, placeholders):
    for placeholder, original in placeholders:
        text = text.replace(placeholder, original)
    return text

def lowher(text):
    # Mark special cases
    text, placeholders = mark_special_cases(text)
    
    # Lowercase the remaining text
    text = text.lower()
    
    # Restore special cases
    text = unmark_special_cases(text, placeholders)
    
    return text

if __name__ == "__main__":
    import sys

    # Read input from stdin if no file is specified
    input_text = sys.stdin.read() if len(sys.argv) == 1 else open(sys.argv[1]).read()

    output_text = lowher(input_text)
    print(output_text)
