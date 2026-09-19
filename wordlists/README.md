# Codeword source

`codewords.txt` is the fixed public codebook. It contains exactly 65,536
unique lowercase ASCII words, sorted by byte order. Every entry is 3 to 12
bytes long. Its contents and order must not change after release.

The list is based on Alan Beale's public-domain 12dicts 6.0.2 word lists:

- `International/3of6game.txt` supplies the main vocabulary.
- Words present in both `International/5d+2a.txt` and
  `American/2of12.txt` supply reviewed additions.

Entries rated as likely or possible profanity by `cuss` were excluded. The
remaining additions were reviewed for readability, familiarity and neutral
meaning.

12dicts was compiled by Alan Beale, who explicitly released these lists to
the public domain and requested acknowledgement. `cuss` is MIT licensed,
copyright Titus Wormer.

- 12dicts 6.0.2: <https://wordlist.aspell.net/12dicts/>
- 12dicts documentation: <https://wordlist.aspell.net/12dicts-readme/>
- cuss: <https://github.com/words/cuss>
