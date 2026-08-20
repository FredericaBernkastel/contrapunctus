# The two clef glyphs

`clefs.otf` is a **subset of [Bravura]**, the reference font for [SMuFL], cut
down to the two characters this program draws: `U+E050` the G clef and `U+E062`
the F clef. 15 kB, from 868 kB — the other 3712 glyphs are of no use to a
program that writes three-voice fugues and draws its own accidentals.

Bravura is licensed under the **SIL Open Font License 1.1**, with **Reserved
Font Name "Bravura"** — the full text and Steinberg's copyright notice are in
[`clefs-LICENSE.txt`](clefs-LICENSE.txt), which the licence requires to travel
with the font.

That reserved name is why the subset is called **Contrapunctus Clefs** and not
Bravura. Clause 3 of the OFL says a Modified Version may not use the reserved
name, and a subset is a modified version; renaming it is not a courtesy, it is
the condition on which the font may be redistributed at all.

## Why a font rather than drawn outlines

A clef could be drawn from Bézier paths, as the accidentals in `score.rs` are
drawn from line segments, and that would ship no font at all. Two reasons not to.
Outlines lifted out of a font are still derived from it, so the licence follows
them into the source and is far less clear there than it is around a file plainly
marked as a font. And SMuFL specifies that **one em is four staff spaces** with
each clef's origin **on the line it names** — so a font, drawn at a size of four
staff spaces with its baseline on the G or F line, is placed exactly right by
construction rather than by a constant somebody tuned by eye.

## Regenerating it

```
pip install fonttools
curl -sLO https://raw.githubusercontent.com/steinbergmedia/bravura/master/redist/otf/Bravura.otf
python -c "
from fontTools import subset
opts = subset.Options(); opts.name_IDs = ['*']; opts.name_languages = ['*']
font = subset.load_font('Bravura.otf', opts)
s = subset.Subsetter(options=opts); s.populate(unicodes=[0xE050, 0xE062]); s.subset(font)
for r in font['name'].names:
    if r.nameID in (1, 3, 4, 6, 16, 18): r.string = 'Contrapunctus Clefs'
subset.save_font(font, 'clefs.otf', opts)
"
```

[Bravura]: https://github.com/steinbergmedia/bravura
[SMuFL]: https://w3c.github.io/smufl/
