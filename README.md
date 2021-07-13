# Cliptools

A cross-platform CLI utility to manipulate the clipboard (aka pasteboard).

Supports Windows, MacOS, X11, and Wayland; powered by [copypasta](https://github.com/alacritty/copypasta).

## Status

In development. Tested on MacOS, and I'm working on X11.

## Features

 - Print data from clipboard, optionally for a specific type (`cliptools paste [-t format]`)
 - List types available for current contents of clipboard (`clipboards list-types`)
 - Change contents of clipboard (`clipboard copy [-t format]`)

## Usage

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
by the `-t` / `--type` argument. If you need to use another content type, you can use `--custom-type`.
In this case, you need to know how your platform encodes content types for the clipboard.

In some cases, such as if you use JSON input, cliptools will assume you are using standard aliases,
unless you prefix the content type with an at sign (`@`). For instance, `@image.tiff` would
give you [TIFF](https://en.wikipedia.org/wiki/TIFF) contents on MacOS.

### Return codes

 - 0 if everything went well
 - 1 if data was not found (e.g. no data for the requested format)
 - \>1 for other errors

### TODO

 - Support clipboard history. This is not available on all platforms, but is useful on those for which it is.
