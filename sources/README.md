# Sources

The documents are not kept in the repository (see `.gitignore`). Fetch
them into the directories named to check a reading. All were fetched on
22 and 23 November 2026. The `.txt` files beside the PDFs were made with
`pdftotext -layout`.

| Directory | Document | From | Used for |
| --- | --- | --- | --- |
| `revised-report-1962/` | P. Naur (ed.), "Revised Report on the Algorithmic Language ALGOL 60", *CACM* 6(1), January 1963, 1–17 (`algol60-cacm.pdf`, a scan); and an HTML transcription (`report.htm`) | https://www.cs.tufts.edu/~nr/cs257/archive/peter-naur/algol60.pdf, https://www.masswerk.at/algol60/report.htm | Everything. The procedures of `examples/report-procedures.alg` were checked against page 14 of the scan. The HTML transcription has slips in that section (`Spur` lacks `Result: (s)`, `Innerproduct` lacks `begin real s;`), so the scan is the authority |
| `modified-report-1976/` | De Morgan, Hill and Wichmann, "Modified Report on the Algorithmic Language ALGOL 60" (HTML) | https://www.masswerk.at/algol60/modified_report.htm | The input and output procedures |
| `knuth-man-or-boy-1964/` | D. E. Knuth, "Man or boy?", *Algol Bulletin* 17.2.4, July 1964, page 7 (`A17P7.GIF`, a scan) | https://archive.computerhistory.org/resources/text/algol/algol_bulletin/A17/A17P7.GIF | `examples/man-or-boy-1964.alg`. Knuth conjectured −121 for k = 10, "probably wrong"; the answer is −67 |
| `kruseman-aretz-x1-2003/` | F. E. J. Kruseman Aretz, *The Dijkstra–Zonneveld ALGOL 60 compiler for the Electrologica X1*, CWI note SEN-N0301, 2003 (338 pages) | https://gtoal.com/languages/algol60/x1algol/Electrologica_X1.pdf (also https://ir.cwi.nl/pub/4155) | The X1 dialect: the 23 properties of section 1.3, the stops of appendix A, the library of chapter 7, and the program of appendix B (`historical/jaz164.alg`) |
| `kruseman-aretz-x1-x8/` | F. E. J. Kruseman Aretz, *A comparison between the ALGOL 60 implementations on the Electrologica X1 and the Electrologica X8* (182 pages) | https://gtoal.com/languages/algol60/x1algol/Electrologica-X1-vs-X8.pdf (also https://ir.cwi.nl/pub/13677) | The representation of values (3.2) and the run-time for statement (3.3.3) |
| `gtoal-x1algol/` | Graham Toal's text rebuilds: `x1algol.pas` (Kruseman Aretz's Pascal version of the X1 compiler) and `x8algol.a60` (the X8 compiler, in ALGOL 60, with underlined word delimiters) | https://gtoal.com/languages/algol60/x1algol/ | The X1 library names; `x8algol.a60` as a large test of the parser (it parses completely; it needs the X8's own library to go further) |
| `dijkstra-mr35/` | E. W. Dijkstra, "ALGOL-60 Translation", Mathematical Centre report MR 35 (a scan without text) | https://www.cs.utexas.edu/~EWD/MCReps/MR35.PDF | Not read yet |
| `sergev-x1/` | Serge Vakulenko's *x1-algol-compiler* (git clone, commit 938fa03, 27 July 2025, Apache-2.0): the X1 compiler in Pascal and C, a simulator with its own reconstruction of the run-time, a library, 22 example programs, and a translation of section 7 of the X8 library manual | https://github.com/sergev/x1-algol-compiler | The library layouts (`doc/library-procedures.md`), the input data of `historical/jaz164.in`, and the comparison in `docs/x1.md`. Read, not built |
| `van-den-hove-2019/` | The public parts of G. van den Hove d'Ertsenryck, *New Insights from Old Programs: The Structure of The First ALGOL 60 System*, PhD thesis, University of Amsterdam, 2019: front matter, contents, appendices A (the ALGOL reports) and B (bibliography), back matter | https://dare.uva.nl/id/8ecd6e9b-767d-44ab-84a7-8eed35822da5 | The contents: chapter III, part 2 is "Executing ALGOL 60 Programs" |
| `cwi-x1-programs/` | Programs for the X1 from the CWI software archive: C. H. A. Koster's jungle generator (1964 or earlier) and L. Meertens's Quinio (1964), with photographs of the listings | https://github.com/cwi-software-archive/Algol60 | See `docs/x1.md` |

## Not found

- The X1 run-time complex and Kruseman Aretz's X1 emulator, which he
  describes in [KA03, 1.6].
- The X1 user manual itself, which would give the layouts of `print`,
  `FLOT`, `FIXT` and `ABSFIXT`.
- The rest of van den Hove's thesis, held back for a book. A full copy
  at core.ac.uk sits behind a bot check and was not fetched; the author's
  page http://www.fibonacci.org/GHE3 did not answer.
