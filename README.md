<div align="center">
<h2><code>oO0OoO0OoO0Oo CiTY oO0OoO0OoO0Oo</code></h2>
</div>

![preview](/pics/preview.gif?raw=true)

Just a moving city ~~ASCII~~ _ANSI escape sequence_ art  
Inspired by redis `lolwut version 6` command:

![redis](/pics/lolwut.png?raw=true)

### Usage and fine tuning

- Check `--help` for usage: you might want to use `-a` instead of setting canvas size manually
- For larger canvas sizes, it may be difficult for your terminal to render the city without fps drops or "tearing". [Alacritty](https://github.com/alacritty/alacritty) offers probably the most smooth rendering, even when target fps is set to 120
- Also if you're willing to build it yourself (`cargo build --release`), you can change layer count or tune some of their parameters (colors, density, speed) in `main.rs` (look for `LayerDesc` structures)

