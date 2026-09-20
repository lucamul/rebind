#set page(width: 4in, height: 6in, margin: 0.5in)
#set text(font: "Helvetica", size: 12pt)

= Chapter One

This is the first paragraph on the first page. It exists to prove that ordinary body text extracts correctly, in reading order, with a normal font.

This is a second paragraph, on the same page, so paragraph-boundary detection has something to work with.

#emph[This whole line is italic, to prove font-based style detection has a real signal to read.]

This paragraph is deliberately long so that it overflows across the page boundary without any page break command, forcing the layout engine to split it mid sentence exactly where a real book's justified body text would split naturally, which is exactly the case the page break classifier needs to get right: recognizing that nothing deliberate happened here, that the words before the boundary and the words after the boundary are really one continuous thought that a human reader would never notice was split across two different pages at all, and that treating this boundary as if it were a meaningful new section would be exactly the kind of mistake the whole point of this project is to avoid making silently, so the paragraph keeps going a while longer still, well past where the page is going to run out of room, on purpose.

This is a short paragraph right after the long one, still on whichever page the long one ends up finishing on, with nothing special about the boundary between them either.

#pagebreak()
= Chapter Two

This paragraph opens a deliberate new chapter, with a heading immediately above it and a fresh page underneath it — the clean, unambiguous case a page-break classifier should get right with high confidence.

#v(2em)
#align(center)[
*A Short Poem*

#emph[Line one of the poem,] \
#emph[line two of the poem,] \
#emph[centered and italic,] \
#emph[so verse detection has a fixture.]
]
