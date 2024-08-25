import io
import re

import spacy

# Compile regex pattern once
CODE_BLOCK_PATTERN = re.compile(r"(```[\s\S]*?```|`[^`]*`)")

# Load the spaCy model (consider using a smaller model if available)
nlp = spacy.load("en_core_web_sm")


def mark_code_blocks(text):
    code_blocks = CODE_BLOCK_PATTERN.findall(text)
    placeholders = [f"__CODE_BLOCK_{i}__" for i in range(len(code_blocks))]
    for i, code_block in enumerate(code_blocks):
        text = text.replace(code_block, placeholders[i])
    return text, placeholders, code_blocks


def unmark_code_blocks(text, placeholders, code_blocks):
    for placeholder, code_block in zip(placeholders, code_blocks):
        text = text.replace(placeholder, code_block)
    return text


def process_text(text):
    doc = nlp(text)
    output = io.StringIO()

    for token in doc:
        if token.ent_type_ in ["PERSON", "ORG", "GPE"] or token.text.isupper():
            output.write(token.text_with_ws)
        else:
            output.write(token.text.lower() + token.whitespace_)

    return output.getvalue()


def lowher(text):
    text, placeholders, code_blocks = mark_code_blocks(text)
    processed_text = process_text(text)
    final_text = unmark_code_blocks(processed_text, placeholders, code_blocks)
    return final_text


if __name__ == "__main__":
    default_input_text = """
This is an EXAMPLE of Proper Noun Detection. `Inline code should STAY`.
And this should stay inside a ```python
code block
```. More TEXT.
"""
    input_text = default_input_text

    output_text = lowher(input_text)
    print(output_text)
