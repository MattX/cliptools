# Cliptools

A cross-platform CLI utility to manipulate the clipboard (aka pasteboard).

Supports Windows, MacOS, X11, and Wayland; powered by [arboard](https://github.com/ArturKovacs/arboard)
(or rather a fork; bugs are my fault!)

## Usage

To copy data from the terminal:

```
$ echo abc | cliptools copy
```

To paste the contents of the clipboard as HTML:

```
$ cliptools paste -t html
<p>Assertions are always checked in both debug and release builds, and cannot
be disabled. See <a href="https://doc.rust-lang.org/std/macro.debug_assert.html" title="debug_assert!">
<code>debug_assert!</code></a> for assertions that are not enabled in release builds by default.</p>
```

To view types supported by the current clipboard selection:

```
$ cliptools list-types
@public.tiff
html
```

## Status

In development. Tested on MacOS, and I'm working on X11. There should be basic support for copy
and paste (but no requesting specific types, etc) on Wayland and Windows.

## Features

 - Print data from clipboard, optionally for a specific type (`cliptools paste [-t format]`)
 - List types available for current contents of clipboard (`clipboards list-types`)
 - Change contents of clipboard (`clipboard copy [-t format]`)

### Content types

Clipboards generally support storing the same piece of information as different formats. For instance,
if you copy HTML content from a browser, the browser might save two pieces of information in the
clipboard, as part of one logical item:
 - An HTML item, containing the HTML markup for the selection, for pasting into a rich-text editor
 - A plain text item, containing the stripped text that was copied, for pasting into a terminal
   or plain text editor.

Each application can request the content type it prefers from the clipboard.

The specific way content-types are encoded depends on the platform, so cliptools provides standard
aliases that work cross-platform: `url`, `html`, `pdf`, `png`, `rtf`, and `text`. These are accepted
by the `-t` / `--type` argument. If you need to use another content type, you can use `--system-type`.
In this case, you need to know how your platform encodes content types for the clipboard.

In some cases, such as if you use JSON input, cliptools will assume you are using standard aliases,
unless you prefix the content type with an at sign (`@`). For instance, `@image.tiff` would
give you [TIFF](https://en.wikipedia.org/wiki/TIFF) contents on MacOS.

### Return codes

 - 0 if everything went well
 - 1 if data was not found (e.g. no data for the requested format), or there was an error setting
   clipboard contents.
 - \>1 for other errors

### TODO

 - Support clipboard history. This is not available on all platforms, but is useful on those for
   which it is.
 - If using JSON input to fill in several content-types, there is no way to send in binary data (that's a
   JSON format limitation). There should be another input method for this use case.
