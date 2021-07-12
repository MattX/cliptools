# Cliptools

A cross-platform CLI utility to manipulate the clipboard (aka pasteboard).

Supports Windows, MacOS, X11, and Wayland; powered by [copypasta](https://github.com/alacritty/copypasta).

## Status

In development. Tested on MacOS, and I'm working on X11.

## Features

 - Print data from clipboard, optionally for a specific type (`cliptools paste [-t format]`)
 - List types available for current contents of clipboard (`clipboards list-types`)
 - Change contents of clipboard (`clipboard copy [-t format]`)

### Usage

#### Return codes

* 0 if everything went well
* 1 if data was not found (e.g. no data for the requested format)
* \>1 for other errors

### TODO

 - Support clipboard history. This is not available on all platforms, but is useful on those for which it is.
