## [3.3.0](https://github.com/crazyscot/brot3/releases/tag/v3.3.0) - 2026-01-31

### 🚀 Features

- Initial implementation of perturbation mode (for Mandelbrot, exp=2) - ([cf54c7d](https://github.com/crazyscot/brot3/commit/cf54c7d1d70a265983afe5a329810e3e297b3e16))
- Update colourers collection - ([1340701](https://github.com/crazyscot/brot3/commit/1340701c0cbc4e3353d71b4488cf315179f38778))
- Add iteration-culling mode - ([af91d4e](https://github.com/crazyscot/brot3/commit/af91d4e6f0ded4aace1f80d99cbd302459ea384b))

### 🐛 Bug Fixes

- Make WhiteFade and BlackFade look more like their originals; vectorise - ([4379bc3](https://github.com/crazyscot/brot3/commit/4379bc31c159bd1259864500e30bf7dc598edbe2))
- Buffer limit for very high iteration limit - ([8d2f523](https://github.com/crazyscot/brot3/commit/8d2f523dbd4a763659b3f0e7599d46f2e4410375))
- Refactor fractal distance, Filaments to work around numeric precision limits - ([e298ebf](https://github.com/crazyscot/brot3/commit/e298ebfc58b24cec35662f7b957b170ea5f27bdb))

### 📚 Documentation

- Update workspace structure diagram - ([47371f7](https://github.com/crazyscot/brot3/commit/47371f7a62737ae6620d127501b115552bd76ed9))

### ⚡ Performance

- Allow BigVec/BigComplex arithmetic taking a reference to the second argument - ([d001c14](https://github.com/crazyscot/brot3/commit/d001c146dbf6f5faf6a7ac1b7500e9facf0b3c9b))
- Don't run inspector unless actually asked to - ([c0749d3](https://github.com/crazyscot/brot3/commit/c0749d35c8c1aef4eeb86cd592785ec1ef3f39a7))

### 🚜 Refactor

- Move BigComplex and BigVec2 into shader - ([2c8c5ed](https://github.com/crazyscot/brot3/commit/2c8c5ed5e63f77ac3958e6378293c6445f6a5dcb))
- Merge shader_common into shader - ([a77fc02](https://github.com/crazyscot/brot3/commit/a77fc02e3769751103cbcd8ad96f22008b8960ef))
- Expose exponent monomorphisation macro - ([4b8faac](https://github.com/crazyscot/brot3/commit/4b8faac2c8a84b561f175670fba5c2b285cb50a7))

### 🧪 Testing

- Fill in coverage gaps in BigComplex and BigVec2 - ([aa18e69](https://github.com/crazyscot/brot3/commit/aa18e69cb789d1a4b295e0da05b36bf0cf35c744))
- Consolidate & expand colour known answer tests - ([e2a4777](https://github.com/crazyscot/brot3/commit/e2a4777897bfd44f2501597bb0afabe0f43769d4))
- Fill in coverage in exponentiation - ([edafb80](https://github.com/crazyscot/brot3/commit/edafb80c37c1922c3197551bdf5e9174694be84b))

### 🏗️  Build, packaging & CI

- Add output to ui build to reduce confusion - ([e162d47](https://github.com/crazyscot/brot3/commit/e162d4702a007192d23a968f27a57edc2f9ce398))

### ⚙️ Miscellaneous Tasks

- *(ci)* Dependabot config - ([af6283b](https://github.com/crazyscot/brot3/commit/af6283bc8c1b719c7d8411578da8cfb735206d3f))
- CheckableButton: add indeterminate ability - ([63f230f](https://github.com/crazyscot/brot3/commit/63f230f8f0e6867025c7bb705912190d73a24390))
- Smarter macro use in fractal::render() - ([bc14a76](https://github.com/crazyscot/brot3/commit/bc14a76914b6b5e2230c702067d670cc02789a0d))
- Add BigComplex conjugate(), recip(), div<FBig> - ([2f9cb5f](https://github.com/crazyscot/brot3/commit/2f9cb5f80fabd4157de03a4d1d29fb1985064cea))
- Rename make_complex to make_bigcomplex for legibility - ([6cd92e2](https://github.com/crazyscot/brot3/commit/6cd92e2d1578b494fe49593e6c22195a6f133e94))
- Remove dead code in PushExponent, fill in its test coverage - ([3d13bff](https://github.com/crazyscot/brot3/commit/3d13bff467122ab84b6ed48d3256dd365fcb6841))

## [3.2.0](https://github.com/crazyscot/brot3/releases/tag/v3.2.0) - 2025-12-09

### 🚀 Features

- *(cli)* Add --cache-size override, enable wrap_help - ([0e1079e](https://github.com/crazyscot/brot3/commit/0e1079e9cfe6e8c2d7ed46b458af55f2b8a3e58d))
- Add modifier keys (shift, alt) to affect control speed - ([680cd2a](https://github.com/crazyscot/brot3/commit/680cd2a7871e15fa24321a5e438c6ba49981cb86))
- Option to always reiterate, without using the fractal data cache - ([ab9f5b5](https://github.com/crazyscot/brot3/commit/ab9f5b5a6e839d542f3b5c862d5ee8402e73310a))
- Add undocumented performance test mode, accessed by ctrl+F11 - ([0fd7b99](https://github.com/crazyscot/brot3/commit/0fd7b99993f3a8d77c107b03f9a1fa60f7ff5cb0))

### 🐛 Bug Fixes

- *(build)* Update the internal reference to spirv-builder - ([74be54b](https://github.com/crazyscot/brot3/commit/74be54b1f8dace308afe9f842c264e0697b0d035))
- *(macos)* PNG file rendering in UI - ([50e58d7](https://github.com/crazyscot/brot3/commit/50e58d72a03b7697232fe7034baea191849acf87))
- Align exponentiation special cases, add tests - ([647864d](https://github.com/crazyscot/brot3/commit/647864d92d2fb6e27e2081709a4937325c525fc6))
- Derivative calculation in distance estimator - ([8b8456c](https://github.com/crazyscot/brot3/commit/8b8456c7e0fb616334a9c88dfff25c2f0c50ddee))
- Avoid NaNs breaking filaments styles - ([a440310](https://github.com/crazyscot/brot3/commit/a440310751e7a040abd4f8f36bc8550fb90e74c8))
- Update inspector data when requested via the context menu - ([b1d7493](https://github.com/crazyscot/brot3/commit/b1d74931c0310d044652a2b5adeed1bd16268e48))
- Menu icon transparency - ([e44dfc6](https://github.com/crazyscot/brot3/commit/e44dfc658f144bd100282632a359d330f60a1d96))
- Shader checks cache buffer size and falls back appropriately - ([a1f8881](https://github.com/crazyscot/brot3/commit/a1f88817b4f73907d8671c897995e7ad24869594))
- Don't let --fullscreen CLI option get lost - ([f480431](https://github.com/crazyscot/brot3/commit/f480431f425dcf1241d3a4f20ac67d925b5825a9))
- Make keyboard zooms more consistent with mouse scrolls - ([92beeaf](https://github.com/crazyscot/brot3/commit/92beeaf9ca220e0577c6714bf2eaca4c360ba8ba))

### ⚡ Performance

- Simplify shader by moving log2 calculation into Exponentiator and making it branchless - ([687e3ec](https://github.com/crazyscot/brot3/commit/687e3eccd11b977d924797150c80e012264c68e9))
- Make exponentiation special cases branchless - ([69abbda](https://github.com/crazyscot/brot3/commit/69abbda68620758a39bb84f7f5511789db396b3e))
- Merge the individual fractal traits - ([f5e214d](https://github.com/crazyscot/brot3/commit/f5e214d815ca95c6e62581bace02b44bae8e49f8))
- Remove some branching in colourspace - ([512d88f](https://github.com/crazyscot/brot3/commit/512d88fcca75cf89852418f04d4f0e3eb7392385))
- Refactor Exponentiator - ([2b23e95](https://github.com/crazyscot/brot3/commit/2b23e9541f5b29bc78ac156a608e6f69859c2b39))

### 🚜 Refactor

- ColourStyle::None becomes a Colourer - ([75ded55](https://github.com/crazyscot/brot3/commit/75ded5537fb3062c35ed90ebcc27af7985d19bf6))
- RgbVec -> Vec3Rgb; ditch Rgb type - ([a14ca7c](https://github.com/crazyscot/brot3/commit/a14ca7c60a583698f653154e8a44a47d2ede3a43))
- Unsplit PointResult, request the actual buffer size we want from wgpu - ([3ee3f23](https://github.com/crazyscot/brot3/commit/3ee3f238885e7869aedf8f01f02040fa355382ff))
- GridRef/GridRefMut check their buffer limits - ([7f18572](https://github.com/crazyscot/brot3/commit/7f185728ba031111ee547952276eaeeaa348a07e))
- Set the cache buffer size at runtime (largest available monitor) - ([e6b5073](https://github.com/crazyscot/brot3/commit/e6b50731c92c1c3a1efb3d89e9f9856ca4bddf01))
- Fullscreen, so it works better on macOS - ([ba3e7c5](https://github.com/crazyscot/brot3/commit/ba3e7c515c54f309218ffd90a28f4b239038ed61))
- Merge shader_util crate into shader - ([84d9afd](https://github.com/crazyscot/brot3/commit/84d9afd6a7bdf8e35180141109092f399ee375ac))

### 🎨 Styling

- Create options menu - ([255723f](https://github.com/crazyscot/brot3/commit/255723fe4f4296f7fdfcf90fd788c1fa4a3c3630))

### 🧪 Testing

- Use nextest in coverage, clone doctests to maintain coverage - ([0ec6d21](https://github.com/crazyscot/brot3/commit/0ec6d213308c03aeff9f3585576ca790b2e502a0))
- Improve coverage in shader - ([bebda73](https://github.com/crazyscot/brot3/commit/bebda736c4f893025f57fab57293eb72d5927591))
- Add benchmarks for colour conversions - ([dfd15ec](https://github.com/crazyscot/brot3/commit/dfd15ec8dba4f80b4cf951af1023abe3de189ae1))

### 🏗️  Build, packaging & CI

- *(!)* Change default feature flags to NOT include runtime compilation - ([b7d69e3](https://github.com/crazyscot/brot3/commit/b7d69e32168973b5722c624e5cd23a159006a37a))
- Re-enable OSX builds - ([d4e8f97](https://github.com/crazyscot/brot3/commit/d4e8f97429ee07cfbebe552b0141d2aa83108a8a))
- Use later easy-shader-runner; updates rust-gpu - ([df0bcdd](https://github.com/crazyscot/brot3/commit/df0bcdd93238198cff2c5579830340df830aea48))
- Add more checks, streamline - ([8e01907](https://github.com/crazyscot/brot3/commit/8e01907835b16f070c1c304cb7655e1fa86450a0))

### ⚙️ Miscellaneous Tasks

- *(macos)* Update keyboard shortcuts - ([e5ecb25](https://github.com/crazyscot/brot3/commit/e5ecb250907b149334cf2e17d5495d73591feda7))
- Add division to BigVec2 - ([765f959](https://github.com/crazyscot/brot3/commit/765f95912c488fc426a1c17fc9b46fa0edf9203b))
- Use mimalloc as global allocator - ([b8d642e](https://github.com/crazyscot/brot3/commit/b8d642e7a6519eb477e5c8c799635f1db8037fbb))
- Introduce flag_if() helper - ([b68b0fa](https://github.com/crazyscot/brot3/commit/b68b0fab619b035d7f4d6a82615f790075f5ccde))
- Redistribute data between PointResultA and PointResultB - ([6c4388f](https://github.com/crazyscot/brot3/commit/6c4388faa395688101e26bdc4e70bab4422593bc))
- Add scripts/janitor - ([e7f23ae](https://github.com/crazyscot/brot3/commit/e7f23ae38611db20fce260381c04dcc22edd426d))

## [3.1.2](https://github.com/crazyscot/brot3/releases/tag/v3.1.2) - 2025-11-15

### 🏗️  Build, packaging & CI

- Fix release auto-building - ([e149fc9](https://github.com/crazyscot/brot3/commit/e149fc9a76cd388374f6d09b4738d265829f1861))

## [3.1.1](https://github.com/crazyscot/brot3/releases/tag/v3.1.1) - 2025-11-15

### 🐛 Bug Fixes

- *(build)* Move prebuilt shader download into target/ - ([f97ab22](https://github.com/crazyscot/brot3/commit/f97ab225f5cd876aab62b77062aa521dbe0e4713))
- *(ci)* Artifact removal failure is not a build failure - ([b88e019](https://github.com/crazyscot/brot3/commit/b88e019bda796d1a2b8935765e9c42bee894a452))

## [3.1.0](https://github.com/crazyscot/brot3/releases/tag/v3.1.0) - 2025-11-15

### 🚀 Features

- *(maths)* Support complex powers - ([d50c528](https://github.com/crazyscot/brot3/commit/d50c5280463fe03f4e2bebd82e4c2efca77724b2))
- *(shader)* Calculate distance estimate, add to PointResult - ([2a6d488](https://github.com/crazyscot/brot3/commit/2a6d488a0a333dbbd308e452f149e063d9ffa6f4))
- Point inspector (via context menu) - ([d1d8a76](https://github.com/crazyscot/brot3/commit/d1d8a76fd506d72590db9d17f31b260a0b8c75f6))
- Brightness & Saturation palette modifier styles - ([f3d4948](https://github.com/crazyscot/brot3/commit/f3d49489823e6016610b4d91f5e525ce51b80e32))

### 🐛 Bug Fixes

- *(build)* Rebuild-if-changed logic - ([d994971](https://github.com/crazyscot/brot3/commit/d994971fc7a4d15405af82752587fcdfd42c569f))
- *(build)* Double "-dirty" in version string - ([560e726](https://github.com/crazyscot/brot3/commit/560e7262b148b4c8b659179b0e19cd6ec237de34))
- *(ci)* Toolchain for cargo-deny - ([c1e8422](https://github.com/crazyscot/brot3/commit/c1e842292d0f26837f2625f9c9229cb6ae3a6f96))
- Fractional iterations calculation - ([70e5721](https://github.com/crazyscot/brot3/commit/70e5721dbbb7cba5ab412abba003dc7dc60df078))
- Store fractional iters as range 0..1 to reduce error terms - ([04511d0](https://github.com/crazyscot/brot3/commit/04511d0dc94d7f0a631c6c1fed76c21ab0889028))
- Remove render data discontinuety at the escape radius - ([3272735](https://github.com/crazyscot/brot3/commit/327273513bc15745f80664456d53d20e30664c1d))
- Default PushExponent - ([496c501](https://github.com/crazyscot/brot3/commit/496c5017551bfbcb3951f9257ab97eda45c7710c))
- Use correct derivative of exponential term in distance estimate of higher power fractals - ([533e9ef](https://github.com/crazyscot/brot3/commit/533e9ef192dc8338cc83eef5ff4d0bf9e2f9a10f))
- Special case exponentiation to avoid shader aborts - ([cb73349](https://github.com/crazyscot/brot3/commit/cb733496ae4db6df4ed54fd8ae4fb2dae6eed160))
- Special case the fractional escape count to avoid subnormals with small exponents - ([565ec45](https://github.com/crazyscot/brot3/commit/565ec4526cd5ab3433a72d73470923371d766913))
- Signedness of complex exponents - ([3ba343c](https://github.com/crazyscot/brot3/commit/3ba343cbd9fda0133055a8e2d3fdb839c0c90512))

### 📚 Documentation

- Remove no-longer-relevant changelog entries - ([59e95f9](https://github.com/crazyscot/brot3/commit/59e95f93a92959718eb4767a80184d22832274e0))
- Freshen git-cliff config - ([a7b36bd](https://github.com/crazyscot/brot3/commit/a7b36bdbb153a0e490a02475fac8f3adc1984901))
- Improve changelog generation - ([3c60609](https://github.com/crazyscot/brot3/commit/3c606099d3f2288d4b371c6aa23410a9f26dbbe6))
- Update readme - ([d0fd563](https://github.com/crazyscot/brot3/commit/d0fd56321ea3fedc0f74c7c74ef04cb283f0dae0))
- Create workspace-structure.dot, workspace-structure.png - ([c17adb5](https://github.com/crazyscot/brot3/commit/c17adb5f85e8a2bd2b5e26a8549fca29ca2a4ce6))

### 🚜 Refactor

- *(colour)* Move the inside check inside of each colouring algorithm - ([e6a7a19](https://github.com/crazyscot/brot3/commit/e6a7a194dbecd620364790f81fc0394fbb232f59))
- *(shader_util)* Re-organise arbitrary precision structs, feature-gate them. Tidy up docs. - ([1fac335](https://github.com/crazyscot/brot3/commit/1fac335452ebcee9e22c2a5b27b597c7fc524779))
- Rename big_complex crate to util - ([28f690c](https://github.com/crazyscot/brot3/commit/28f690cb76940480f8344e9462bb1911e2f372d2))
- Replace Fractional iters checkbox with Render Style drop-down - ([295f0bd](https://github.com/crazyscot/brot3/commit/295f0bdce1e2e1cd86d7389a58128e2b8b3c4f2b))
- Remove 'inside' from PointResult - ([7359b88](https://github.com/crazyscot/brot3/commit/7359b889f9f8fe94995d663f915618e8d00bd948))
- PointResult & bind group - ([b744d32](https://github.com/crazyscot/brot3/commit/b744d32e62b36e2932670cbfc93e1fea6d6f631b))
- Merge Bool flags in FragmentConstants into a bitfield - ([91e6de3](https://github.com/crazyscot/brot3/commit/91e6de36e3c4630720977f92f877acd894177380))
- Rename RenderStyle to ColourStyle - ([9bc6447](https://github.com/crazyscot/brot3/commit/9bc6447aa3125e6c6dccc1d1420ebb42820aa6a5))
- Move ColourStyle into Palette - ([2509f54](https://github.com/crazyscot/brot3/commit/2509f545a471b014bb8e8f1b840ac4964667dec4))
- Move enums from shader_common into an enums submodule - ([6d999ce](https://github.com/crazyscot/brot3/commit/6d999ce7874baaca35679cbba4fd8e1bec658515))
- Move PointResult structs out to their own module - ([9fc8984](https://github.com/crazyscot/brot3/commit/9fc8984ebe1dfed337b7aa307bfcadd0f81e62f5))
- Setters for Palette - ([c61e927](https://github.com/crazyscot/brot3/commit/c61e927a58cfc9367d6e9c2fef78cbc0ab8bfe02))
- Pivot colouring algorithms to return HSV - ([70dc9ac](https://github.com/crazyscot/brot3/commit/70dc9ac363a02ef66663ad796801a56ff2101f92))
- Colour Style becomes Continuous, Discrete, None - ([063b08c](https://github.com/crazyscot/brot3/commit/063b08c7ea72855e890c75d6d071d17f43f9d295))
- Use macro to generate the enum boilerplate - ([5b2bc9e](https://github.com/crazyscot/brot3/commit/5b2bc9ea8b138c8ed2a56cfd16900157edc0eaac))
- Rename Palette.style -> colour_style - ([8a15fe8](https://github.com/crazyscot/brot3/commit/8a15fe817aa727411aabcb7839043666d1455ddb))

### 🎨 Styling

- Turn off Esc-to-exit - ([34f444e](https://github.com/crazyscot/brot3/commit/34f444e95bfd0aec0b71fe33854f01eff00f8860))
- Neaten keyboard help window - ([eb40968](https://github.com/crazyscot/brot3/commit/eb409686fa1e88e6c9163d23f69439b0a214f897))
- Improve data read-out - ([f3a4591](https://github.com/crazyscot/brot3/commit/f3a45917139a791034698e73635f64af44ce23f3))
- Tidy up Inspector - ([c69cf49](https://github.com/crazyscot/brot3/commit/c69cf4934bd87ba9f4f43b52bc5fbe3e50db1b2a))

### 🧪 Testing

- Re-enable GridRef tests with the updated CI config - ([1362a06](https://github.com/crazyscot/brot3/commit/1362a061238b9acd439d4843fe82242bc029d53b))

### 🏗️  Build, packaging & CI

- Remove duplicate jobs - ([65e98f0](https://github.com/crazyscot/brot3/commit/65e98f08ad001545f2296390ac8662471a9870c3))
- Update rerun-if-changed markers - ([1c987f0](https://github.com/crazyscot/brot3/commit/1c987f0b4e838ddca56acd1ac2ec988816f33aff))
- Don't overwrite an unchanged built.rs; fix .git/HEAD check - ([69fe6d3](https://github.com/crazyscot/brot3/commit/69fe6d3b2bb407b807805385183636f6d24797ae))
- Coverage runs with --locked-frozen --release - ([fc89275](https://github.com/crazyscot/brot3/commit/fc89275016cc41567d1c25e047a0710f6ec77015))
- Move release profile to default settings for now - ([f7e3b74](https://github.com/crazyscot/brot3/commit/f7e3b74dea7cc410d39e1cfa6c536ca3587d555c))
- Overhaul workflow - ([fc35d6e](https://github.com/crazyscot/brot3/commit/fc35d6e957e8951870111ca54288ca824ce6398d))
- Fix consistency of test/doc/doctest/bench declarations - ([09ecc77](https://github.com/crazyscot/brot3/commit/09ecc77457e338395df0162c7ed724cf2f722075))
- Set up cargo deny, move cargo fmt to a separate job - ([696463d](https://github.com/crazyscot/brot3/commit/696463d4fc3b869adfced6d0176ebaf3fdbb59db))
- More tweaks - ([de63296](https://github.com/crazyscot/brot3/commit/de632969e72c3140753e071a99910afae026bfd4))
- The ui crate is not a library - ([2627c17](https://github.com/crazyscot/brot3/commit/2627c17af52f4132be6a346882f1be9a13c424d5))
- Update release workflow: always run publish - ([a480476](https://github.com/crazyscot/brot3/commit/a480476d6fa6b7bb9a7c32bf80e3a2043e0cfdef))
- Unify platform jobs - ([b6a25b9](https://github.com/crazyscot/brot3/commit/b6a25b98490ec7706bd1d946866fe8e2de948d7c))
- Make save-cache inputs less confusing - ([dbf0ce4](https://github.com/crazyscot/brot3/commit/dbf0ce49ad1fd95358831ac3523d2c56c554d08f))
- Switch off mac port for now as non functional - ([df40f71](https://github.com/crazyscot/brot3/commit/df40f7142a1b58ead0f0844eae7f811afea54a69))
- Add cargo-machete - ([1428c9a](https://github.com/crazyscot/brot3/commit/1428c9ac8e3bddc40e1cc9761e04b135dbf1ceb8))

### ⚙️ Miscellaneous Tasks

- *(maths)* Complex exponentiator - ([740996c](https://github.com/crazyscot/brot3/commit/740996c8de8202c5e794a1f85a18acba4a107465))
- *(maths)* Expand PushExponent to support complex powers - ([6390eda](https://github.com/crazyscot/brot3/commit/6390eda4e0f3f56f378c87bae5924c9d1a441382))
- *(shader)* Calculate pixel spacing, pass to colour_data - ([cc183a6](https://github.com/crazyscot/brot3/commit/cc183a6801cfbea1a06744d25af6c0909ba8c2aa))
- *(ui)* Change meaning of F2 (Show/Hide UI) to mean the main UI only - ([5dd9318](https://github.com/crazyscot/brot3/commit/5dd93186f993cd7f2a3a7fa9c8324c42fdfd47a2))
- *(windows)* Run without console - ([c5f4773](https://github.com/crazyscot/brot3/commit/c5f47733d9b596a635b11cc5447b1ce8352ff3d8))
- Update to abels_complex@0.3.0 - ([41fd22c](https://github.com/crazyscot/brot3/commit/41fd22ca8e437f0dd4ba7731a5d823b383eedb92))
- Detangle BigComplex and BigVec2 from shader_util - ([dc6b7eb](https://github.com/crazyscot/brot3/commit/dc6b7ebc5912af2f35b00fc421e177eb46fe08a4))
- Introduce dynamic formatting function for floats - ([259f168](https://github.com/crazyscot/brot3/commit/259f168eedd6f6d72a6a6f2a56492e951fe4b8d0))
- Mark shader-common enums as non-exhaustive, to avoid UB potential - ([36620b3](https://github.com/crazyscot/brot3/commit/36620b32780ef638221b58ca3b67ead6bcf8d0ba))
- Gather point final angle and distance from origin - ([3d069d6](https://github.com/crazyscot/brot3/commit/3d069d6c0810433f595e5016e96122cc68d68d81))
- Derive NoUninit on more members of shader_common and shader_util - ([097b907](https://github.com/crazyscot/brot3/commit/097b9072f9b1645ccdfc1b1566fa54091962c199))
- Drop unused Bool struct - ([36a677a](https://github.com/crazyscot/brot3/commit/36a677a91bcca340dee370c14063ada21d7b87b7))
- Add more colour space reprs/conversions - ([3574b0c](https://github.com/crazyscot/brot3/commit/3574b0c619e17509732be8a94119eb7a0d2dd08e))
- Add fractal debug output macro - ([da0b6f3](https://github.com/crazyscot/brot3/commit/da0b6f3d4ae5dcdaca25a8ef98e765926c72e8b6))
- Move iters calculation into PointResult - ([4931cc6](https://github.com/crazyscot/brot3/commit/4931cc66f9b962757ddda2bb749bf93ca48edbdc))
- Set more sensible defaults for FragmentConstants - ([64b4018](https://github.com/crazyscot/brot3/commit/64b4018ebd6d428c5a5ba07aadcef51cb539fe94))
- Aesthetic tweaks: default maxiter, LCH gradient - ([b575af1](https://github.com/crazyscot/brot3/commit/b575af101bac00970048b2d16f25475fdf22f570))
- Move the pixel size calculations to all be in the same place - ([d681989](https://github.com/crazyscot/brot3/commit/d681989fd29478c593d725b9e9621a2bb6eece1f))
- Deduplicate constants - ([b5d7c76](https://github.com/crazyscot/brot3/commit/b5d7c76f2c346b10cb1eab70cdad612a2b0b38cd))
- Apply a human-facing zoom factor so that the initial view size is apparent zoom factor 1.0 - ([8a7cc5e](https://github.com/crazyscot/brot3/commit/8a7cc5e483b5b8271783903bbae2f6f89a839df0))
- Rework the user-presented version string, include in window title, add --version - ([bc2533b](https://github.com/crazyscot/brot3/commit/bc2533be9602d3699d94cc66f038682f2768855b))
- Built.rs writes a version file for CI - ([6b9b7cd](https://github.com/crazyscot/brot3/commit/6b9b7cd32356cd37013d70cc3662a77be156d533))

## [3.0.0](https://github.com/crazyscot/brot3/releases/tag/v3.0.0) - 2025-10-24

### 🚀 Features

- [**breaking**] Complete rewrite. The engine now runs as a GPU shader, with the UI written using egui.
  Older changelog entries has been discarded as no longer relevant.
