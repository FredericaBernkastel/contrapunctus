# The music glyphs

`music.otf` is a **subset of [Bravura]**, the reference font for [SMuFL], cut
down to the four characters this program needs: `U+E050` the G clef, `U+E062` the
F clef, and the flat and the sharp — `U+E260` and `U+E262`, each also mapped to
its own Unicode character, `U+266D` and `U+266F`. 16 kB, from 868 kB.

Bravura is licensed under the **SIL Open Font License 1.1**, with **Reserved
Font Name "Bravura"** — the full text and Steinberg's copyright notice are in
[`music-LICENSE.txt`](music-LICENSE.txt), which the licence requires to travel
with the font.

That reserved name is why the subset is called **Contrapunctus Music** and not
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

## Why the accidentals are here, and why they are moved

`score.rs` draws its own accidentals against noteheads and does not want these.
They are here for **text**: a compass label has to be able to write `E♭4`, and no
font egui ships has U+266D or U+266F in it. What that produced was a tofu box in
the panel, which is how it was found. There is no natural, because nothing asks
for one: the labels are given a key signature's alterations, and a key signature
has no naturals in it.

They are the one thing in this subset that is **not** SMuFL-placed. SMuFL centres
an accidental on the notehead it applies to, so the glyph straddles the baseline
— correct on a staff, and in a line of text it reads as a subscript. The command
below raises each one by its own depth so it sits on the baseline like a letter:
the sharp by 0.348 em and the flat by 0.175. The clefs are not touched, because a
clef hanging below its baseline is exactly what puts its origin on the line it
names.

`glyph::an_accidental_sits_on_the_baseline_and_a_clef_does_not` asserts both
halves, so a regenerated subset that forgets the shift fails rather than looking
slightly wrong to somebody.

## Regenerating it

```
pip install fonttools
curl -sLO https://raw.githubusercontent.com/steinbergmedia/bravura/master/redist/otf/Bravura.otf
python -c "
from fontTools import subset
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.transformPen import TransformPen
from fontTools.misc.transform import Offset

CLEFS = [0xE050, 0xE062]
ACCIDENTALS = {0x266D: 0xE260, 0x266F: 0xE262}

opts = subset.Options(); opts.name_IDs = ['*']; opts.name_languages = ['*']
font = subset.load_font('Bravura.otf', opts)
s = subset.Subsetter(options=opts)
s.populate(unicodes=CLEFS + list(ACCIDENTALS.values())); s.subset(font)

cmap, glyphs = font.getBestCmap(), font.getGlyphSet()
top = font['CFF '].cff.topDictIndex[0]
for smufl in ACCIDENTALS.values():
    name = cmap[smufl]
    b = BoundsPen(glyphs); glyphs[name].draw(b)
    pen = T2CharStringPen(font['hmtx'][name][0], glyphs)
    glyphs[name].draw(TransformPen(pen, Offset(0, -b.bounds[1])))
    top.CharStrings[name] = pen.getCharString(private=top.Private)

for t in font['cmap'].tables:
    if t.isUnicode():
        for uni, smufl in ACCIDENTALS.items():
            if smufl in t.cmap: t.cmap[uni] = t.cmap[smufl]

for r in font['name'].names:
    if r.nameID in (1, 3, 4, 6, 16, 18): r.string = 'Contrapunctus Music'
subset.save_font(font, 'music.otf', opts)
"
```

[Bravura]: https://github.com/steinbergmedia/bravura
[SMuFL]: https://w3c.github.io/smufl/
