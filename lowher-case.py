import re

import spacy


def mark_code_blocks(text):
    code_block_pattern = re.compile(r"(```[\s\S]*?```|`[^`]*`)")
    code_blocks = code_block_pattern.findall(text)
    placeholders = [f"__CODE_BLOCK_{i}__" for i in range(len(code_blocks))]
    for i, code_block in enumerate(code_blocks):
        text = text.replace(code_block, placeholders[i])
    return text, placeholders, code_blocks


def unmark_code_blocks(text, placeholders, code_blocks):
    for i, placeholder in enumerate(placeholders):
        text = text.replace(placeholder, code_blocks[i])
    return text


def process_text(text):
    # Load the spacy model for proper noun detection
    nlp = spacy.load("en_core_web_sm")
    doc = nlp(text)

    processed_text = []

    for token in doc:
        if token.ent_type_ in ["PERSON", "ORG", "GPE"] or token.text.isupper():
            processed_text.append(token.text_with_ws)
        else:
            processed_text.append(token.text.lower() + token.whitespace_)

    return "".join(processed_text)


def lowercase_except_nouns_acronyms_and_code(text):
    text, placeholders, code_blocks = mark_code_blocks(text)
    processed_text = process_text(text)
    final_text = unmark_code_blocks(processed_text, placeholders, code_blocks)
    return final_text


# Example usage
input_text = """
This is an EXAMPLE of Proper Noun Detection. `Inline code should STAY`.
And this should stay inside a ```python
code block
```. More TEXT.
"""
output_text = lowercase_except_nouns_acronyms_and_code(input_text)
print(output_text)
