import { defineConfig, rustdoc } from "sourcey";

export default defineConfig({
  name: "itertools Rust API",
  siteUrl: "https://raw.githack.com",
  baseUrl: "/rohitmulani63-ops/itertools/docs/sourcey-rust-api/docs/sourcey-itertools/site",
  prettyUrls: false,
  repo: "https://github.com/rust-itertools/itertools",
  editBranch: "master",
  theme: {
    preset: "default",
    colors: {
      primary: "#7c2d12",
      light: "#c2410c",
      dark: "#431407"
    }
  },
  navigation: {
    tabs: [
      {
        tab: "Rust API",
        slug: "rust-api",
        source: rustdoc({
          manifest: "./adapter/Cargo.toml",
          crates: ["itertools"],
          snapshot: "./snapshots/rustdoc.json",
          mode: "snapshot",
          toolchain: "nightly-2025-11-22",
          features: { all: true },
          sourceBasePath: "../../../",
          doctestsIndex: true
        })
      }
    ]
  },
  navbar: {
    links: [
      { label: "Upstream repository", href: "https://github.com/rust-itertools/itertools" },
      { label: "docs.rs", href: "https://docs.rs/itertools/" }
    ]
  },
  footer: {
    links: [
      { label: "Source at cef9a820", href: "https://github.com/rust-itertools/itertools/tree/cef9a8203ec485b06ec1889550cde0aa989d103a" }
    ]
  }
});
