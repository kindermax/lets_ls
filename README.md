# Lsp server for lets task runner

## Supported features

* [x] Goto definition
  - Navigate to definitions of `mixins` files
* [x] Completion
  - Complete commands in `depends`
* [ ] Diagnostics
* [ ] Hover
* [ ] Document highlight
* [ ] Document symbol
* [ ] Formatting
* [ ] Signature help
* [ ] Code action

## Development

Build:

```bash
cargo build
```

Test:

```bash
cargo test
```

Lint:

```bash
cargo clippy
# or to fix lints
cargo clippy --fix --bin "lets_ls"
```

## Release build

```bash
cargo build --release
```

## Integration with Neovim

Add new filetype:

```lua
vim.filetype.add({
  filename = {
    ["lets.yaml"] = "yaml.lets",
  },
})
```

In your `neovim/nvim-lspconfig` servers configuration:

In order for `nvim-lspconfig` to recognize lets_ls we must define config for `lets_ls`

```lua
require("lspconfig.configs").lets_ls = {
  default_config = {
    cmd = { 
      "lets_ls",
    },
    filetypes = { "yaml.lets" },
    root_dir = util.root_pattern("lets.yaml"),
    settings = {},
  },
}
```

And then enable `lets_ls` in then `servers` section:

```lua
return {
  "neovim/nvim-lspconfig",
  opts = {
    servers = {
      lets_ls = {},
      pyright = {},  -- just to show an example how we enable lsp servers
    },
  },
}
```

## Integration with VSCode

Extension can be found [here](https://marketplace.visualstudio.com/items?itemName=kindritskyimax.vscode-lets).