"""Convert DEMorphy morphological dictionary to Harper's Rune format.

Reads DE_morph_dict.txt and produces:
  - dictionary_de.dict: lemma-based word list with property flags
  - annotations_de.json: property definitions and basic affix rules

Strategy: Extract unique lemmas from DEMorphy and categorize them by POS.
Also include common inflected forms that are very different from their lemma
(e.g., irregular verb forms). The compound splitter handles compound words
at runtime, so we don't need to store every compound.
"""

import json
import sys
import os
from collections import defaultdict


def parse_demorphy(path):
    """Parse DE_morph_dict.txt.

    Returns:
      - word_forms: dict of word_form -> list of (lemma, analysis_str)
      - lemma_analyses: dict of lemma -> set of POS tags seen
    """
    word_forms = defaultdict(list)
    lemma_info = defaultdict(set)  # lemma -> set of "POS,features"
    current_word = None

    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.rstrip('\n')
            if not line:
                continue

            parts = line.split(' ', 1)
            if len(parts) == 1:
                current_word = line.strip()
            elif current_word is not None:
                lemma = parts[0]
                analysis = parts[1]
                word_forms[current_word].append((lemma, analysis))
                lemma_info[lemma].add(analysis)

    return word_forms, lemma_info


def get_pos_flags(analyses):
    """Given a set of analysis strings, return property flags and gender."""
    flags = set()
    gender = None

    for analysis in analyses:
        parts = analysis.split(',')
        if not parts:
            continue
        pos = parts[0].strip()
        features = [p.strip() for p in parts[1:]]

        if pos == 'NN':
            flags.add('N')
            if gender is None:
                if 'masc' in features:
                    gender = 'masculine'
                elif 'fem' in features:
                    gender = 'feminine'
                elif 'neut' in features:
                    gender = 'neuter'
        elif pos == 'NNP':
            flags.add('N')
            flags.add('Q')
        elif pos == 'V':
            flags.add('V')
        elif pos == 'ADJ':
            flags.add('A')
        elif pos == 'ART':
            flags.add('T')
        elif pos in ('PREP', 'PREPART', 'POSTP'):
            flags.add('P')
        elif pos == 'CONJ':
            flags.add('C')
        elif pos in ('PRP', 'DEMO', 'INDEF', 'POSS', 'REL', 'WPRO'):
            flags.add('R')
        elif pos == 'PRTKL':
            flags.add('X')
        elif pos in ('CARD', 'ORD'):
            flags.add('M')

    return flags, gender


def should_include_lemma(lemma):
    """Filter lemmas for the dictionary."""
    if not lemma:
        return False
    if lemma.replace('-', '').replace('.', '').isdigit():
        return False
    if '(TRUNC)' in lemma:
        return False
    # Skip single characters except common ones
    if len(lemma) == 1 and lemma not in ('a', 'à', 'ä', 'o', 'ö', 'u', 'ü', 'i', 'O', 'A'):
        return False
    return True


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    demorphy_path = os.path.join(script_dir, 'demorphy-data', 'DE_morph_dict.txt')

    if not os.path.exists(demorphy_path):
        print(f"ERROR: {demorphy_path} not found.")
        print("Run: cd scripts && git clone https://github.com/DuyguA/german-morph-dictionaries.git demorphy-data")
        sys.exit(1)

    output_dir = os.path.join(script_dir, '..')
    dict_path = os.path.join(output_dir, 'dictionary_de.dict')
    annot_path = os.path.join(output_dir, 'annotations_de.json')

    print("Parsing DEMorphy dictionary...")
    word_forms, lemma_info = parse_demorphy(demorphy_path)
    print(f"  Found {len(word_forms)} unique word forms, {len(lemma_info)} unique lemmas")

    # We'll build the dictionary from two sources:
    # 1. All lemmas (base forms) with their POS flags
    # 2. All inflected word forms that differ from their lemma
    #    (so spell-check accepts "Häuser" not just "Haus")
    #
    # This gives us full coverage without needing affix rules.
    # To keep size manageable, we include all forms but use a compact format.

    print("Building word list from all word forms...")
    entries = {}  # word -> (flags_str, gender)

    for word_form, analyses_list in word_forms.items():
        if not should_include_lemma(word_form):
            continue

        # Collect all analyses for this word form
        all_analyses = set()
        for lemma, analysis in analyses_list:
            all_analyses.add(analysis)

        flags, gender = get_pos_flags(all_analyses)
        flags_str = ''.join(sorted(flags))

        if word_form in entries:
            # Merge flags
            old_flags, old_gender = entries[word_form]
            merged_flags = ''.join(sorted(set(old_flags) | set(flags_str)))
            entries[word_form] = (merged_flags, old_gender or gender)
        else:
            entries[word_form] = (flags_str, gender)

    # Also ensure all lemmas are included
    for lemma, analyses in lemma_info.items():
        if not should_include_lemma(lemma):
            continue
        if lemma not in entries:
            flags, gender = get_pos_flags(analyses)
            flags_str = ''.join(sorted(flags))
            entries[lemma] = (flags_str, gender)

    # Sort for deterministic output
    sorted_entries = sorted(entries.items(), key=lambda x: (x[0].lower(), x[0]))

    print(f"  Total entries: {len(sorted_entries)}")

    # Write dictionary file
    print(f"Writing {dict_path}...")
    with open(dict_path, 'w', encoding='utf-8') as f:
        f.write(f"{len(sorted_entries)}\n")
        for word, (flags, gender) in sorted_entries:
            if flags:
                f.write(f"{word}/{flags}\n")
            else:
                f.write(f"{word}\n")

    # Write annotations file - properties only, no affix rules
    print(f"Writing {annot_path}...")
    annotations = {
        "affixes": {},
        "properties": {
            "N": {
                "metadata": {
                    "noun": {}
                }
            },
            "V": {
                "metadata": {
                    "verb": {}
                }
            },
            "A": {
                "metadata": {
                    "adjective": {}
                }
            },
            "D": {
                "metadata": {
                    "adverb": {}
                }
            },
            "T": {
                "metadata": {
                    "determiner": {}
                }
            },
            "P": {
                "metadata": {
                    "preposition": True
                }
            },
            "C": {
                "metadata": {
                    "conjunction": {}
                }
            },
            "R": {
                "metadata": {
                    "pronoun": {}
                }
            },
            "X": {
                "metadata": {}
            },
            "Q": {
                "metadata": {
                    "noun": {
                        "is_proper": True
                    }
                }
            },
            "M": {
                "metadata": {}
            }
        }
    }

    with open(annot_path, 'w', encoding='utf-8') as f:
        json.dump(annotations, f, indent='\t', ensure_ascii=False)
        f.write('\n')

    # Print stats
    noun_count = sum(1 for _, (f, _) in sorted_entries if 'N' in f)
    verb_count = sum(1 for _, (f, _) in sorted_entries if 'V' in f)
    adj_count = sum(1 for _, (f, _) in sorted_entries if 'A' in f)

    file_size = os.path.getsize(dict_path)
    print(f"\nStats:")
    print(f"  Total entries: {len(sorted_entries)}")
    print(f"  Nouns: {noun_count}")
    print(f"  Verbs: {verb_count}")
    print(f"  Adjectives: {adj_count}")
    print(f"  Dictionary file size: {file_size / 1024 / 1024:.1f} MB")
    print("Done!")


if __name__ == '__main__':
    main()
