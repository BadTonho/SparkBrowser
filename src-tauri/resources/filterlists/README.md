# Bundled filter lists

Snapshots of EasyList, EasyPrivacy and NoCoin are stored here for offline startup.
Their original headers and license notices must be preserved. See the tracking
protection documentation for sources, update behavior and coverage.

## Licenses and attribution

These files are third-party data. They are **not** covered by Spark's own license
and keep the licenses of their authors. Spark reads them as data and does not
modify them; they are separate files, not part of Spark's source code.

| File | Source | License |
| --- | --- | --- |
| `easylist.txt` | [EasyList](https://easylist.to/) by The EasyList authors | GNU GPL v3 or later and Creative Commons Attribution-ShareAlike 3.0 (see below) |
| `easyprivacy.txt` | [EasyPrivacy](https://easylist.to/) by The EasyList authors | GNU GPL v3 or later and Creative Commons Attribution-ShareAlike 3.0 (see below) |
| `nocoin.txt` | [NoCoin Filter List](https://github.com/hoshsadiq/adblock-nocoin-list/) by Hosh | MIT, see [`NOCOIN-LICENSE.txt`](./NOCOIN-LICENSE.txt) |

- EasyList and EasyPrivacy: the [EasyList license page](https://easylist.to/pages/licence.html)
  says the files may be used under either license, while the project's
  [`package.json`](https://github.com/easylist/easylist/blob/master/package.json) declares
  `(GPL-3.0-or-later AND CC-BY-SA-3.0)`. Spark takes the stricter reading: the files are kept
  unmodified, with their original headers and credit to the authors, so both sets of terms are met.
  The full license texts are available at
  [gnu.org/licenses/gpl-3.0](https://www.gnu.org/licenses/gpl-3.0.html) and
  [creativecommons.org/licenses/by-sa/3.0](https://creativecommons.org/licenses/by-sa/3.0/).
- Each file's header (`! Title`, `! Homepage`, `! Licence`) must stay intact when the
  snapshot is refreshed.
- Credit: EasyList and EasyPrivacy are maintained by [The EasyList authors](https://easylist.to/).
