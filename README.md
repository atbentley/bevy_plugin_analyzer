# bevy_plugin_analyzer

Use rust-analyzer to statically inspect a bevy plugin crate and find all implementations of `Component`.

## Usage

```
$ cargo run --example sample
PluginCrate {
    name: "sample_plugin",
    components: [
        PluginComponent {
            name: "Point",
            path: "sample_plugin::Point",
            fields: [
                "x",
                "y",
            ],
        },
    ],
}
```
