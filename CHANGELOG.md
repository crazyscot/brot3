## [3.6.0](https://github.com/crazyscot/brot3/releases/tag/v3.6.0) - 2026-06-21

### 🚀 Features

- *(cli)* Add --exponent - ([759bc3c](https://github.com/crazyscot/brot3/commit/759bc3c39f4c7d4dac228bd5638189b739167e05))
- *(ui)* F5 shows/hides the FPS count - ([2933bf6](https://github.com/crazyscot/brot3/commit/2933bf6bd31cf4cd947062d8c9df3e8ead541600))
- Make PNG compression setting a user-facing option, set sensible default - ([c3924b7](https://github.com/crazyscot/brot3/commit/c3924b77a06a26b1650e9dd12afe96a89565d9fe))

### 🐛 Bug Fixes

- *(debug)* Apply correct timestamp resolution to debug output - ([6b68149](https://github.com/crazyscot/brot3/commit/6b681497a67c7a7bdf3420d35634a1f07d80ce22))
- *(load)* Allow JSON fields to be missing (they will default) - ([ca0151c](https://github.com/crazyscot/brot3/commit/ca0151cc02a1e5f0f99de25bd2503296875a07aa))
- *(macos)* Use Command in menu accelerators - ([5802b80](https://github.com/crazyscot/brot3/commit/5802b8016612ed0381a7563013bf6788473ec486))
- Use correct resolution on compute shader timestamps - ([ea4e007](https://github.com/crazyscot/brot3/commit/ea4e007b792d5d652d3e8355482f8fadc34a1b8d))
- Inspector assertion failure in perturbation mode - ([3f77009](https://github.com/crazyscot/brot3/commit/3f77009b2f590ff1d6607c6c427a65875ac9a792))

### 📚 Documentation

- Create AGENTS.md, copilot-instructions.md - ([81daf08](https://github.com/crazyscot/brot3/commit/81daf081f66030a136e9b75ab36d94865cb3b83b))

### ⚡ Performance

- *(colour)* Reduce branching around colourer offset processing - ([8624662](https://github.com/crazyscot/brot3/commit/8624662a279c76521d06450ff36cc6a3ad1f9d30))
- *(colour)* Refactor pixel short-cutting - ([fbe8207](https://github.com/crazyscot/brot3/commit/fbe8207d81d22d40f4d2b69f9b97ee9fba359e18))
- *(colour)* Merge & debranch finish_colour - ([7168808](https://github.com/crazyscot/brot3/commit/716880847ba43bf2270a28d182855463c56f15b9))
- *(colour)* Merge colour_simple_family and colour_powered_family - ([f6d96da](https://github.com/crazyscot/brot3/commit/f6d96daf68e4dd8c074693efa28b356010990241))
- *(easy-shader-runner)* Add ability to suppress rendering - ([5d3e16c](https://github.com/crazyscot/brot3/commit/5d3e16cef7d380332d3e28da1f2a089ad62fcbf6))
- *(fractal)* Remove bitwise short-cut operations where it makes sense to in the shader - ([27c26ef](https://github.com/crazyscot/brot3/commit/27c26ef7923ef1053c47439bd7a55fc644bfae29))
- *(fractal)* Use loop..break instead of while in the shader hot loop - ([bc41a29](https://github.com/crazyscot/brot3/commit/bc41a29ff35c104e3c0a6e44a95e98134bc35309))
- *(fractal)* Remove exponentiator from RunningConsts when not needed - ([d2daa53](https://github.com/crazyscot/brot3/commit/d2daa533551240295433edda748fbe672a48af63))
- *(fractal)* Gate perturbation-mode items out of RunningConstants/RunningVariables - ([29d7569](https://github.com/crazyscot/brot3/commit/29d75693058252a9aa4c62c9240d46d9d7795b4a))
- *(fractal)* Remove Algorithm from RunningConsts when not needed - ([ce2dbbc](https://github.com/crazyscot/brot3/commit/ce2dbbc956951359454592bb9deb6e9f9ac4a299))
- *(fractal)* Remove z from RunningVariables - ([3060bec](https://github.com/crazyscot/brot3/commit/3060beca3d2e2b714d343d259914da2428dd297a))
- *(fractal)* Refactor to remove a branch in calculating the smoothed iteration count - ([db65fdb](https://github.com/crazyscot/brot3/commit/db65fdb7e444b47e03737f1fa3b0e8c3f619e369))
- *(fractal)* Small fractal optimisations - ([6251935](https://github.com/crazyscot/brot3/commit/6251935fac51ed0bf87ea70af6eb9c6c682a78b6))
- *(fractal)* Optimise iteration cull for spirv - ([988fdbe](https://github.com/crazyscot/brot3/commit/988fdbe66f120d3a81f5b3f5e1698b4f51aa23a4))
- *(fractal)* Reduce hot-loop branching in perturbation mode - ([f577165](https://github.com/crazyscot/brot3/commit/f577165858c2c60524ebe452cc695aa38bf7ace0))
- *(fractal)* Cache the next refpoint ahead of time - ([e0b9fcd](https://github.com/crazyscot/brot3/commit/e0b9fcdd4d61ccb43a386c64f211a3378e2a7321))
- *(hot-reload)* Build only the needed shader up front, background the others - ([6de4049](https://github.com/crazyscot/brot3/commit/6de40495e591336a308127616e490df4d72d1006))
- *(shader)* Refactor inspector draw to be branchless - ([942b39f](https://github.com/crazyscot/brot3/commit/942b39f8a3c24f7accb05bcb23d09482863614f2))
- Improve PNG write speed - ([7bcf0c8](https://github.com/crazyscot/brot3/commit/7bcf0c83fe9c5f41031f665de3e51e1689324f27))
- Dynamically change shader given the current UI state - ([72b28bf](https://github.com/crazyscot/brot3/commit/72b28bf0e133919b36368aafe153fb2652820ed9))
- Gather timestamps on shader render if available - ([4b9a51c](https://github.com/crazyscot/brot3/commit/4b9a51c165c6c215035bb957d1cb9b60a5b11311))
- Optimize mandelbrot_perturbed_iterate_algorithm (slight gain) - ([a1989e4](https://github.com/crazyscot/brot3/commit/a1989e4ecd9d9d00bedbb53514671d671ced4719))
- Feature-gate distance-estimate logic in shader - ([4f1cd82](https://github.com/crazyscot/brot3/commit/4f1cd821620eb5b31a54928f64105a88e01ebc79))
- Streamline complex multiplication in the perturbation mode hot loop - ([1495b1f](https://github.com/crazyscot/brot3/commit/1495b1fd04d217b9545523b37dd642353e692b85))
- Refactor norm_sqr out of RunningVariables - ([1b612cb](https://github.com/crazyscot/brot3/commit/1b612cbfee4d9639c963a06b98a9cd2b41309d42))
- Remove boundary from RunningVariables when not needed - ([eb72560](https://github.com/crazyscot/brot3/commit/eb72560a664eaf5c1dfe60a4fd84fc7cb40e25a5))
- Refactor save logic to remove unnecessary copies of pixel data - ([b4c90ba](https://github.com/crazyscot/brot3/commit/b4c90ba37368577071960dbf164c83bb03bdaed9))
- Use a streamed writer when writing PNGs - ([8a81af6](https://github.com/crazyscot/brot3/commit/8a81af6cb105ce5cb7aedb6c390f5c4efb8a6c7b))
- Improve UI responsiveness by suppressing render unless we need to reiterate - ([6cd5b97](https://github.com/crazyscot/brot3/commit/6cd5b972b6da3baf70901ec682f11ada9e94fbfe))

### 🚜 Refactor

- Use winit Modifiers to improve reliability of keyboard events - ([9c1b76a](https://github.com/crazyscot/brot3/commit/9c1b76a094d28bc537d3c6a201cf82d8e3f84fc6))
- Rework and coalesce the colourers - ([6cae3b4](https://github.com/crazyscot/brot3/commit/6cae3b4fd376a430fde573e9a65b058db16d7a41))
- Create RENDER_BYTES_PER_PIXEL const - ([5454fc0](https://github.com/crazyscot/brot3/commit/5454fc07334e4ce1a327c11214adf9d0d0531823))

### 🎨 Styling

- Add perturbation calculation metadata to data read-out - ([a107256](https://github.com/crazyscot/brot3/commit/a107256d17edc4f7cc793792af17680bb6dfbfd7))
- Indicate that iterations are culled in the inspector read-out - ([973af65](https://github.com/crazyscot/brot3/commit/973af65c87aa4db477e1e7a87466245f5d84a09a))
- Reorder debug menu - ([361f304](https://github.com/crazyscot/brot3/commit/361f304fb52cb24d61bb46d86232412d6ce4ed8b))

### 🧪 Testing

- Improve unit test coverage - ([75304d2](https://github.com/crazyscot/brot3/commit/75304d279b972403764ea9b0d584664c4480cc2d))
- Add some integration tests - ([a514de6](https://github.com/crazyscot/brot3/commit/a514de60197e7ff38ea139390c20f5d3ec8b9af7))
- Validate saved data files - ([94b98b4](https://github.com/crazyscot/brot3/commit/94b98b42864f7a4f567ad502aa2e24821dcf06bf))

### 🏗️  Build, packaging & CI

- Add cargo-shear to checks job - ([8fe78c3](https://github.com/crazyscot/brot3/commit/8fe78c3f445bc763b32dd59a9e5a2c422ce8adbe))
- Add all-colourers feature gate - ([dee14c9](https://github.com/crazyscot/brot3/commit/dee14c94b0babcc16d6fef96e3c3109c52799388))
- Add all-fractals feature gate - ([6cadb9f](https://github.com/crazyscot/brot3/commit/6cadb9f9aa619c8f0cedd05d926a4bef2c03244e))
- Add feature gates: variable-exponent, perturbation-mode, standard-mode - ([9c87392](https://github.com/crazyscot/brot3/commit/9c873920ffed44678eb89d5a1aa545ae1ba116b4))
- Multiple shader flavours - ([a3da4ce](https://github.com/crazyscot/brot3/commit/a3da4ce2ae82914bd412e41152176d16940394c1))
- Add headless build - ([ff95e4d](https://github.com/crazyscot/brot3/commit/ff95e4dc62750623993d40fadcd04eae6d256db2))
- Drop the all-colourers feature flag as no longer needed - ([3359b83](https://github.com/crazyscot/brot3/commit/3359b83f1f703693b2b20225644a909f125f7093))
- Run lints on --all-targets - ([30707fa](https://github.com/crazyscot/brot3/commit/30707fa7cd2df6dae48083263a683e21732fa60e))

### ⚙️ Miscellaneous Tasks

- Create ShaderVariant - ([e157830](https://github.com/crazyscot/brot3/commit/e157830f6ae459e220296f02e45a4a64654cdb2a))
- Extend easy-shader-runner to load multiple shaders, selectable at runtime - ([478f61e](https://github.com/crazyscot/brot3/commit/478f61e95bd2422b41e02d06e2264696bfd03eba))
- Remove saturation_style modifier - ([fdd58d0](https://github.com/crazyscot/brot3/commit/fdd58d01fb9dfbeb787fc4e7dc7e28e076513db5))
- Remove Monochrome, LogRainbow colourers - ([89070f2](https://github.com/crazyscot/brot3/commit/89070f26603eb23298392f9a57f68aab575e0694))
- Remove palette gamma, saturation, lightness controls as defunct - ([9f1b489](https://github.com/crazyscot/brot3/commit/9f1b489fc1976db7edaf42ba7b3bbecf49def47d))
- Split short-cutting out of Runner.run - ([e1c616e](https://github.com/crazyscot/brot3/commit/e1c616e0e8501eff5d90090792ebe7f2ff7139be))
- Improve legibility of perturbed calculation - ([8beefdb](https://github.com/crazyscot/brot3/commit/8beefdbf489c365c09117257a07936de1be169af))
- Tidy up undocumented engine exports, benchmark feature requirements - ([add96d8](https://github.com/crazyscot/brot3/commit/add96d8bfcc25b79a49563904b20537ac0a178b1))
- When Show Timings is selected, show the FPS window if it's not already hidden - ([1502956](https://github.com/crazyscot/brot3/commit/15029569ac7af3b5e6f68abdbec8ea25b8b845aa))
- Remove vestigial partial-failure handling when saving PNGs - ([c56ad7c](https://github.com/crazyscot/brot3/commit/c56ad7c696ae2afb483218103e5debc9a5c38ca0))
- PNG render timing output - ([cb0b7e4](https://github.com/crazyscot/brot3/commit/cb0b7e4c335896e28c113598973aa600e0e00a5a))
- Disallow viewport movement while saving - ([b2037fb](https://github.com/crazyscot/brot3/commit/b2037fb48fcaa5a16e07d5613ca85c347a842686))

## [3.5.0](https://github.com/crazyscot/brot3/releases/tag/v3.5.0) - 2026-05-24

### 🚀 Features

- *(!)* Remove Complex power support as the maths was not valid - ([9db6dd0](https://github.com/crazyscot/brot3/commit/9db6dd0971b601d507966ba86bc42102cdb4717e))
- *(!)* Remove support for exponents below 2 - ([a2a5bae](https://github.com/crazyscot/brot3/commit/a2a5bae5f7610bcd9879cd47e1d07eea731b7189))

### 🐛 Bug Fixes

- *(build)* Runtime shader compilation - ([544a149](https://github.com/crazyscot/brot3/commit/544a149c94fa060b76f0212116350df24369c836))
- *(build)* Build with --no-default-features - ([b7bf2fe](https://github.com/crazyscot/brot3/commit/b7bf2fed72f8ca4c94ba5f30b7b40916649631bb))
- *(build)* Build ui with default features, drop prebuild2 step - ([ffe9051](https://github.com/crazyscot/brot3/commit/ffe9051ab0f7ef326265d23dae5c7e6c16200449))
- *(windows)* When run on the command line, attach to that command line. - ([cfaf00a](https://github.com/crazyscot/brot3/commit/cfaf00aabaa173f96506edcc23496cc42e08b3b6))
- In hot-reload mode, don't use a potentially-stale compute shader - ([ec626d7](https://github.com/crazyscot/brot3/commit/ec626d79ba8946f98be1a624f902e518d2339569))
- Cope better with the range of get_current_texture() responses - ([e6e57fe](https://github.com/crazyscot/brot3/commit/e6e57fe0ac8db170583049587f1b11f70c29c7a3))
- Gui startup crash on -i <file> - ([d854b40](https://github.com/crazyscot/brot3/commit/d854b4017cfb5eab9bb094315b367353c2322999))

### ⚡ Performance

- Use a compute shader to render PNG files - ([326f1dd](https://github.com/crazyscot/brot3/commit/326f1dd547cda8142cc50c40417e8eb7701eed3f))
- Reduce cost of the short-circuit checks - ([dc726de](https://github.com/crazyscot/brot3/commit/dc726de9f34d5c4a22713f32a885e106e76d1ab3))
- Ditch the point cache - ([243b0e2](https://github.com/crazyscot/brot3/commit/243b0e262a0a9037a42dbedf08638971cdeec6fb))
- Make log(log(escape threshold)) a compile-time constant - ([454c07a](https://github.com/crazyscot/brot3/commit/454c07a22a68371c76b429dc8a20bc1e7c7bb42b))
- Don't compute expensive distance estimate values unless we need to - ([6aaa8ce](https://github.com/crazyscot/brot3/commit/6aaa8ce639e261a36584b3b5c4b729fab2d4053e))
- Distance estimate calculation - ([e4f9164](https://github.com/crazyscot/brot3/commit/e4f91643ad6e8e331b3cc850069f902e7a1cfcb6))
- Colourer implementations - ([5e03acd](https://github.com/crazyscot/brot3/commit/5e03acd0ff1e849886a94760540f7154ed17b698))

### 🚜 Refactor

- Split out write_png logic from do_save_image - ([0cb216c](https://github.com/crazyscot/brot3/commit/0cb216c5b7de2d8b18a5f0c27d4d9ebd953d7780))
- Do_save_image doesn't need both UiState and FragmentConstants - ([9d00118](https://github.com/crazyscot/brot3/commit/9d001180cd0d16b3594b873cc1a9e68fe04a63b2))
- Move min/max zoom constants and clamping into ViewportZoom - ([c143feb](https://github.com/crazyscot/brot3/commit/c143feb54fe5e9f5c3a59e8af62c291f91662723))
- Seam out rendering from do_save_image - ([2135533](https://github.com/crazyscot/brot3/commit/213553346cd2fd826d027a1b26e67f6e6252d8bb))
- Replace the parallel flag in do_save_image with an enum - ([91fd10e](https://github.com/crazyscot/brot3/commit/91fd10e33317a8e1da80cb4748ecf4a912d11170))
- Move the SPIR-V entrypoints into feat(engine)::entrypoints - ([14acc81](https://github.com/crazyscot/brot3/commit/14acc81fe64d62478329ea4b7b9e3de0dcf6a716))
- Untangle the hot-reload-shader startup code - ([6e66248](https://github.com/crazyscot/brot3/commit/6e662482572b7ce0f9df0f222b6d16505ca4917f))
- Move CPU-based render loops into util - ([2d7a645](https://github.com/crazyscot/brot3/commit/2d7a64598774aa725eecef955b188667ceb1dd32))
- Remove incorrect assumption in render_chunk - ([43de37d](https://github.com/crazyscot/brot3/commit/43de37dd06779c71a5f656e56482e6eb3dd1f78e))
- Reshape point result cache logic to simplify the spirv CFG - ([210c5c2](https://github.com/crazyscot/brot3/commit/210c5c2e82341ee63b615bf70cf045d01ace9903))

### 🎨 Styling

- Update about image, remove unused icon files - ([ef161f5](https://github.com/crazyscot/brot3/commit/ef161f54527945224ae6cbd4ec4c3c2b3a9deb7f))

### 🧪 Testing

- Add whole_frame cycle counting benchmark - ([6dcf822](https://github.com/crazyscot/brot3/commit/6dcf82222a9fcbff2390cd2861205037fae4f2db))
- Add some specific test cases to colourspace conversions - ([3587d54](https://github.com/crazyscot/brot3/commit/3587d54525cd13e69b0cdc52b1551f1f4e6fb939))

### 🏗️  Build, packaging & CI

- Incorporate easy-shader-runner into this repository for convenience - ([bede730](https://github.com/crazyscot/brot3/commit/bede730709e185c95428d761ac35e1c1b0346b39))
- Deny warnings from shader builds - ([c523fff](https://github.com/crazyscot/brot3/commit/c523fff92b5cb5616ba015f93c3461b4522c490c))
- Only run coverage tests on brot3-lib - ([5e8c328](https://github.com/crazyscot/brot3/commit/5e8c328d0a91c6cae4fd68b0743e11141ad7fbcb))
- Update job timeouts - ([bf4f96c](https://github.com/crazyscot/brot3/commit/bf4f96c7ed2157d49e099c74b028bcc3099573a2))
- Update/improve cargo-deny config - ([e4fae41](https://github.com/crazyscot/brot3/commit/e4fae41fe6b21f8f8b8bfc04878f197454700cd1))
- Update upload-rust-binary-action - ([bff44e6](https://github.com/crazyscot/brot3/commit/bff44e6df38e14a5581d9cd3f1bce1e3118801f5))
- Cargo build --verbose - ([c82d4ff](https://github.com/crazyscot/brot3/commit/c82d4ff1dadbf5fe9ed361bbe1c6d09ce1ba0da8))
- Cache-workspace-crates on the shader build - ([eac99b2](https://github.com/crazyscot/brot3/commit/eac99b2721d7834e3b0ef480bfe6c434e87850a1))
- Turn off unnecessary features - ([6db47d5](https://github.com/crazyscot/brot3/commit/6db47d5508134ef9c5ab820b793482721dece881))
- Build multiple UI feature flag combinations - ([d208d2d](https://github.com/crazyscot/brot3/commit/d208d2d2a69a1e9d7c999540daeecfe89611e25c))
- Tidy up rust-toolchain overrides - ([d7e90af](https://github.com/crazyscot/brot3/commit/d7e90af7136ac77054e105235e1c33673a7b3b21))
- Drop the runtime-compilation feature flag - ([1440ec8](https://github.com/crazyscot/brot3/commit/1440ec8d7e732f6ec017d938cbdf78f4592caccc))
- Remove runtime_compile cfg_alias - ([d8b1426](https://github.com/crazyscot/brot3/commit/d8b14263fe1948e32f66bfb3c13b5c8cb644f5c6))
- Remove wasm cfg_alias - ([7075712](https://github.com/crazyscot/brot3/commit/70757120d2ddfb55462d9b0ff9eab3bb2f03e755))
- Refactor away suppress-shader-build feature flag - ([4267291](https://github.com/crazyscot/brot3/commit/426729185c02bcd9e32e729fe4920a9131ce6423))
- Refactor clippy job, check only what matters - ([72103ad](https://github.com/crazyscot/brot3/commit/72103ad47814f37413aa9812b38e6ac0ed9270d5))
- Add use-(compiled,installed)-tools feature flags to crates that use spirv-builder - ([2ca9ae8](https://github.com/crazyscot/brot3/commit/2ca9ae85fee6281cdd8c3aa8501475363aa91d7a))
- Strip debuginfo from release builds to reduce artifact bloat - ([8871447](https://github.com/crazyscot/brot3/commit/88714473cb104d647b2f8df64616c57e6fa14001))
- Split shader build into two jobs - ([639f4df](https://github.com/crazyscot/brot3/commit/639f4df8c59e92b381736a3b603e62947ba11fae))
- Cache the built shader artifact - ([23b3bb4](https://github.com/crazyscot/brot3/commit/23b3bb4bb0abf73ce483641333118339b9aa9461))
- Merge ui_build_prep action back into the main ui job - ([d327d74](https://github.com/crazyscot/brot3/commit/d327d745c3b0d6f7d70a8ade3a48f3181f25589f))
- Consolidate copy-paste toolchain config steps into a copy-paste action - ([5ddd51c](https://github.com/crazyscot/brot3/commit/5ddd51c029e44c2361a365445fab642f2a667daf))
- Combine the two clippy jobs - ([4809d88](https://github.com/crazyscot/brot3/commit/4809d88fa1b82db74c26b5eca5693cc06cb7aa60))
- Add feature flags use-installed-tools, use-compiled-tools - ([36bc200](https://github.com/crazyscot/brot3/commit/36bc200e3c93cc62f86290647d8b4717fa99ca48))
- Refactor ui-hot-reload job to use the prebuilt shader and not build rustc_codegen_spirv - ([e7961b6](https://github.com/crazyscot/brot3/commit/e7961b60383daf094577b9c9df0214ccfa39bccf))
- Don't include debug info on spirv builds - ([829fbfa](https://github.com/crazyscot/brot3/commit/829fbfa3b1037878cad77264b7bcf225d217034e))

### ⚙️ Miscellaneous Tasks

- *(ui)* Remove untested emulate_constants stub feature from compute controller - ([f84a172](https://github.com/crazyscot/brot3/commit/f84a1723ba358e462e43e4b5ea5caea83aa9c864))
- Build fix easy-shader-runner with compute feature flag - ([926cf6b](https://github.com/crazyscot/brot3/commit/926cf6b246f5e61f4b8c42c9160ac41c36824c63))
- Impl UVec2 -> Size conversion - ([355306a](https://github.com/crazyscot/brot3/commit/355306aa26f6730153535e14df01c0d1bb2ff9d4))
- Build fix the runtime_compile feature - ([ef4f8fe](https://github.com/crazyscot/brot3/commit/ef4f8feda8db3d4d02a14d80788dd5f0c958ff04))
- FragmentConstants conversion to UiState - ([e015863](https://github.com/crazyscot/brot3/commit/e0158639df7746d77814b2e8c568f7b982830ca4))
- Add helper for wgpu QuerySets - ([6c8e9d1](https://github.com/crazyscot/brot3/commit/6c8e9d1e344a5e080ffca39c3180e441facb3316))
- Move MAX_MAX_ITERATIONS to root of the ui crate - ([e090a9a](https://github.com/crazyscot/brot3/commit/e090a9a5feb7ace8bbc06a80486548b47df6b43c))
- Tidy up traits and casting, make good use of easy_cast - ([0257505](https://github.com/crazyscot/brot3/commit/0257505c70f96d84daab4edd87a6e8207b072d3d))
- Update rust-gpu to latest (toolchain now nightly-2026-04-11) - ([72fb569](https://github.com/crazyscot/brot3/commit/72fb56977c71600d50180cfd5fe0f03164ceb281))
- Upgrade to wgpu 29, egui 0.34 - ([64088d9](https://github.com/crazyscot/brot3/commit/64088d9dd12753e8cb6ebad7aafbb7411a9018de))
- Clippy - ([7528515](https://github.com/crazyscot/brot3/commit/75285153a789954dad2dd596769e14944a91606d))
- Refactor away cfg-if, we don't need a crate import for that right now - ([24910d9](https://github.com/crazyscot/brot3/commit/24910d9724770bfda06e260937344002b9e285bd))
- Improve GPU feature/limit checking - ([d9e1154](https://github.com/crazyscot/brot3/commit/d9e1154e82627266c5d4b0b25a889a0ccac5540d))
- Vec_sin and vec_cos are no longer necessary - ([4e58566](https://github.com/crazyscot/brot3/commit/4e58566fbf9420ad1428cc09aaeaeeefb34c7672))

## [3.4.0](https://github.com/crazyscot/brot3/releases/tag/v3.4.0) - 2026-04-05

### 🚀 Features

- *(cli)* Add --max_iter, --centre, --zoom and some help headings - ([13458eb](https://github.com/crazyscot/brot3/commit/13458eba3467f5adc92fcb9138b9880ecb04b4a3))
- Create Save Image menu item, file dialog, placeholder save code - ([b610512](https://github.com/crazyscot/brot3/commit/b610512fb04ac73294f5dbe9e0737ecbd64adbf0))
- Save PNG (rendered entirely on the host for now) - ([e4fa01e](https://github.com/crazyscot/brot3/commit/e4fa01eb9e00445e1bea0c669585c21086369e5f))
- Set PNG file descriptive comment and better default filename - ([7bed802](https://github.com/crazyscot/brot3/commit/7bed80265a65c47a038455cb70ff917b5929ede9))
- Add CLI options --colour-style and --brightness-style - ([1d0e437](https://github.com/crazyscot/brot3/commit/1d0e4379427b892995af190f60a160d7d563bb75))
- Save the current UI state as a JSON file - ([0588438](https://github.com/crazyscot/brot3/commit/0588438deeee8d5e65257e9979f6d640d596ef71))
- Include encoded UI state in a PNG, for later reloading - ([e6670a0](https://github.com/crazyscot/brot3/commit/e6670a02d7dfa94e915b0a89624d72757d16aef7))
- CLI to allow loading a saved position file - ([bf33b6d](https://github.com/crazyscot/brot3/commit/bf33b6d574f784f221ef71a261eeef4d76f4e6b2))
- CLI to allow loading a PNG: use its metadata as the initial position - ([93f7898](https://github.com/crazyscot/brot3/commit/93f7898e339aaed0c81f8400ef1b78685d682616))
- Open a file (JSON or PNG) to jump to its position - ([1fe3c71](https://github.com/crazyscot/brot3/commit/1fe3c71e9e95a4a2778e6d71e0ae46ec0e21701e))
- CLI --output to render as PNG - ([fc57ba6](https://github.com/crazyscot/brot3/commit/fc57ba661deed564fe3c9d1fceb1dc058a16b170))

### 🐛 Bug Fixes

- *(build)* Shader on spir-v with debug=true - ([be20e5d](https://github.com/crazyscot/brot3/commit/be20e5df74b4f9bdb147b14b437a72530a446453))
- Negative exponents - ([c134c83](https://github.com/crazyscot/brot3/commit/c134c83f47a88c623b2a489ea4ebf89da0ae4534))
- Mouse zoom gestures should zoom about the current pointer location - ([f5083c6](https://github.com/crazyscot/brot3/commit/f5083c6117bc3449471e814d74def074bfa0cce8))
- Don't include coordinates in suggested filenames, it makes them too long - ([dd0be69](https://github.com/crazyscot/brot3/commit/dd0be690b646cd20e81a4c3b355fc5e4c68dfbd3))
- Anchor data read-out so it doesn't move around on window resize - ([441196b](https://github.com/crazyscot/brot3/commit/441196b181b10d9958a3f207f63af4b741784331))
- Use correct size of perturbation buffer - ([eda56fc](https://github.com/crazyscot/brot3/commit/eda56fc798dc3b2d0e06f40a2d8a28245586291d))
- Apply perturbation mode correctly when given a file at startup - ([9c6f610](https://github.com/crazyscot/brot3/commit/9c6f6104495357fde985f0332b87f9b1886d573a))
- Value clamping in UI - ([1984bfc](https://github.com/crazyscot/brot3/commit/1984bfca3ae0c0c4f3dd418fd127a93021ce2329))
- Buffer overrun that sometimes crashed deep zooms - ([8fc55dd](https://github.com/crazyscot/brot3/commit/8fc55dd43b954c491f16be6933d0a69a80126e7b))
- Exponent hotkeys in integer mode - ([fc6dc4f](https://github.com/crazyscot/brot3/commit/fc6dc4f51969ffab02aae13888c373a20ebbaa66))
- Using some PNG files containing a JSON descriptor as input - ([63ce333](https://github.com/crazyscot/brot3/commit/63ce333ac71331e1fa69eafe5ed0394abb142c46))

### 📚 Documentation

- Doc comments for Algorithm, Colourer, ColourStyle, Modifier - ([080d63e](https://github.com/crazyscot/brot3/commit/080d63e95ac52bcfd992f84a2aba283cfb6fb435))

### ⚡ Performance

- Use rayon on host-side PNG renders for a significant speed-up - ([69d0dbd](https://github.com/crazyscot/brot3/commit/69d0dbd203cc02bd4ba958562f26c06005b918f5))
- Add Exponentiator::apply_power_minus_1_to - ([27b22d9](https://github.com/crazyscot/brot3/commit/27b22d95fe2447708d938ec07926f179c3c1f2f0))
- Don't compute distance estimation unless we actually need to - ([4e62b63](https://github.com/crazyscot/brot3/commit/4e62b639255e30c2c9be97727a30505c1f6a0c24))
- Inline some trait implementations - ([e7456cd](https://github.com/crazyscot/brot3/commit/e7456cdc1ea371ddc374be725059ec5db8e21a77))
- Merge IntegerPower and RealPower structs into Complex - ([fa38340](https://github.com/crazyscot/brot3/commit/fa38340148dd3b7490967ef03c31932376ebb661))
- Optimise macro-generated exponentiation special cases - ([4081f99](https://github.com/crazyscot/brot3/commit/4081f99bae045942888d321a381d87652afd85e4))
- Refactor away warp divergence in colouring algorithms - ([a574a07](https://github.com/crazyscot/brot3/commit/a574a072048ab182b816266cfa727e91719bcf8a))
- Refactor away more if statements to remove warp divergence - ([b17a653](https://github.com/crazyscot/brot3/commit/b17a653ce38a4c6b9cde30ad4f7a9e56a681935c))
- Implement some short-cuts for special cases - ([bcea2ae](https://github.com/crazyscot/brot3/commit/bcea2ae9c9427cec21ce968a2686984db87df27d))

### 🚜 Refactor

- *(!)* Move NumericType and PushExponent into util - ([62a1bd9](https://github.com/crazyscot/brot3/commit/62a1bd9754659b217fa2b90f8adb47b0650dda51))
- *(!)* Merge shader into lib - ([582606d](https://github.com/crazyscot/brot3/commit/582606d0e4034e94c989eea9b7590b03e8ee01f1))
- Add Exponent struct, streamlined for serialization - ([9e74873](https://github.com/crazyscot/brot3/commit/9e74873577a9588501080ab10aaf0e59b488f08c))
- Merge ui's Exponent into PushExponent - ([fa58070](https://github.com/crazyscot/brot3/commit/fa580709bdf94d2663fae7df08e2da1272f76220))
- Move BigComplex and BigVec2 to util - ([f651a31](https://github.com/crazyscot/brot3/commit/f651a31e0a69fd4b2850eae937b7d4914271ebf1))
- UI zoom display calculation - ([55d7ac1](https://github.com/crazyscot/brot3/commit/55d7ac1ffce70690eb88578e0107147e66f837cf))
- Move pixel_spacing_* calculations into a helper trait - ([8b9a62c](https://github.com/crazyscot/brot3/commit/8b9a62c5530cf84813086209eb701937427c70eb))
- Move shader entrypoints into their own module as far as possible - ([ad01333](https://github.com/crazyscot/brot3/commit/ad01333a323e4bedab3f3f1e93c1a86b8603b76d))
- Tidy shader crate exports - ([a791a98](https://github.com/crazyscot/brot3/commit/a791a98052a434264d319e3059a3ed4159ae8bc9))
- Move enums to a new base crate - ([3fd1ea1](https://github.com/crazyscot/brot3/commit/3fd1ea133453d131584f316dd0c87c52b8ef4504))
- Move PushExponent into base - ([a977d63](https://github.com/crazyscot/brot3/commit/a977d6312d992dea57e86f7f13c6b465520d365a))
- Move Exponent from util to base - ([679a6ed](https://github.com/crazyscot/brot3/commit/679a6ed7ef6093674062bc78736de565504e92b6))
- Move Big module into base - ([c7f3e26](https://github.com/crazyscot/brot3/commit/c7f3e26c17ca79ef5379fb8ddb0c7c956e5817c9))
- Move dynfmt into base - ([0969430](https://github.com/crazyscot/brot3/commit/09694305f57deb0765040aaf706f65f2caa89a68))
- Use log instead of dprintln - ([61fb4bc](https://github.com/crazyscot/brot3/commit/61fb4bc1057f6540a374587ce2078861168adc69))
- Move PixelSpacing into base - ([78815e2](https://github.com/crazyscot/brot3/commit/78815e2be12f7a7526694f727aec0b0f18b1180a))
- ViewportZoom - ([d229d0c](https://github.com/crazyscot/brot3/commit/d229d0c0e90f4d2714b9cc917e526cad05ce3542))
- Move FragmentConstants::DEFAULT_SIZE into Controller, as NOMINAL_SIZE - ([738808e](https://github.com/crazyscot/brot3/commit/738808e8122527fb26364187c26cd925d2f4f4c8))
- Calculate LOGLOG2_ESCAPE_THRESHOLD at the start of the run - ([b57c69d](https://github.com/crazyscot/brot3/commit/b57c69dd642afb85ab16a5c2bde90d97f3d861b2))
- Add better float comparisons for Exponent - ([8d31745](https://github.com/crazyscot/brot3/commit/8d31745bf2cc34a0dc8de3e8329e5c9547f95848))
- Rename base to brot3-lib - ([b5111ea](https://github.com/crazyscot/brot3/commit/b5111ea2f2a291394d7180961b16a53a595aa56c))
- Lib::big -> lib::bignum - ([9c40838](https://github.com/crazyscot/brot3/commit/9c4083826f33b30fcedbcdbce7951891643df4d3))
- Internal structure of lib crate - ([216a90f](https://github.com/crazyscot/brot3/commit/216a90f6a084c71172e09a094505a11cd2cc1b62))
- Move relevant UI fields into a serializable structure - ([2a5a433](https://github.com/crazyscot/brot3/commit/2a5a4337ecba5a6eb1e0731875b461a925ffe4cb))
- Move DEFAULT_FRACTAL_PLANE_SIZE and NOMINAL_WINDOW_SIZE into the engine - ([4b20940](https://github.com/crazyscot/brot3/commit/4b20940d960d2a861d281adaa085b3b36b9a53a8))
- Move Controller.size into UiState - ([7564b74](https://github.com/crazyscot/brot3/commit/7564b740a5feb8b0ed0ddb7838510ba348b9788a))
- FragmentConstants.display_string() - ([9f5cd80](https://github.com/crazyscot/brot3/commit/9f5cd8063c259db71efc78e090287e5ea8b54ea4))
- Pivot from anyhow to thiserror in lib, ui, shader_builder - ([ff7189f](https://github.com/crazyscot/brot3/commit/ff7189fd74145eff60638512258cd20010caa4a0))
- Open/save result workflows - ([9db1dad](https://github.com/crazyscot/brot3/commit/9db1dad668c69ff0409bf1723094f67c72dfd7c5))
- Use mpsc channels for task signalling instead of Arc<Mutex<...>> - ([4f2d6fb](https://github.com/crazyscot/brot3/commit/4f2d6fb442665c2e27af959bede7dac5038cf066))
- Move UiState save/load logic into lib - ([35c8806](https://github.com/crazyscot/brot3/commit/35c88062be11c00d155995fd2fbdb72f0548cea4))

### 🎨 Styling

- Move debug options into a submenu - ([ebe95a4](https://github.com/crazyscot/brot3/commit/ebe95a460c59b50078929f50a74a3776317ff062))
- Colourize CLI - ([7f9891c](https://github.com/crazyscot/brot3/commit/7f9891c2293ce53a996c886236531e9b39d7c1e7))
- Add tooltip help text to enum drop-downs in the UI - ([56b41a1](https://github.com/crazyscot/brot3/commit/56b41a1606715e2ef144d67e96d66b9fcd10f755))
- Add a Saving window while saving - ([7daac37](https://github.com/crazyscot/brot3/commit/7daac37ed2d9c701d2d299d4eca145420a12df4c))

### 🧪 Testing

- Add test data for benchmarking, ignore flamegraph output files - ([81d3acd](https://github.com/crazyscot/brot3/commit/81d3acd91d73082dd7a352f553cef137746d1446))
- Add gungraun (cycle-counting) benchmarks - ([f3144c7](https://github.com/crazyscot/brot3/commit/f3144c770b07fdb681e4f83b4c01f016fa8bcf93))
- Add criterion benchmarker - ([4f20331](https://github.com/crazyscot/brot3/commit/4f2033111fb4427038ac8e91c707548fe494d093))
- Merge colour conversion into cycles benchmarker - ([0933600](https://github.com/crazyscot/brot3/commit/0933600028443e8b708190bf679f977e9c47a0ae))
- No cache-workspace-crates on shader - ([df90016](https://github.com/crazyscot/brot3/commit/df900163204daefc1947ad33ebe2f1b4c88a9608))

### 🏗️  Build, packaging & CI

- Add debug to shader_builder, align build options with those of easy-shader-runner - ([053c408](https://github.com/crazyscot/brot3/commit/053c408fbdc240a700fb9625b2187536ecf6fd80))
- Tidy up bench & release profiles - ([891c791](https://github.com/crazyscot/brot3/commit/891c791bb2bf50eea8f011793371de067ff7ac65))
- Debloat release builds (debug = line-tables-only) - ([f12f2dd](https://github.com/crazyscot/brot3/commit/f12f2ddddb017e11450b210147e33670afb93406))
- Move clippy to its own job - ([84a0bc7](https://github.com/crazyscot/brot3/commit/84a0bc71c7e1ab3256311382c472a5812550b513))
- Update actions to versions that use node.js 24 - ([0249dcb](https://github.com/crazyscot/brot3/commit/0249dcbfbaa4f4d6bd6467e4b9769d9fd54d0737))
- Add ui feature flag, enabled by default - ([2f7f081](https://github.com/crazyscot/brot3/commit/2f7f081e7aa736a6ca8dbc33d10cb0b72cf5b778))

### ◀️ Revert

- Refactor: move shader entrypoints into their own module as far as possible - ([219458a](https://github.com/crazyscot/brot3/commit/219458ae950ad4368c859d7011b17c8d0a472157))

### ⚙️ Miscellaneous Tasks

- *(cli)* Rename no-ui to no-controls; tidy up doc comments - ([6fcdfd8](https://github.com/crazyscot/brot3/commit/6fcdfd84810002f756c054f692388b8d54143daf))
- Tidy up cargo config - ([d5c4e6f](https://github.com/crazyscot/brot3/commit/d5c4e6f0846592a2765bb440a0fc5b65bb25beda))
- Add default filename and directory to save dialog - ([89ab14d](https://github.com/crazyscot/brot3/commit/89ab14d532842b204678ffad0fbc04b697e6e82f))
- Add error modal in case the save fails - ([8952508](https://github.com/crazyscot/brot3/commit/8952508ff53894d167229eae9049fca8fb6b5c17))
- Add pre-commit-checks, make it pass - ([ea871b9](https://github.com/crazyscot/brot3/commit/ea871b9c69e498dcebf5587f4c83dfc07883090e))
- Make BigVec2 and BigComplex serialisable (as significand and exponent) - ([9654096](https://github.com/crazyscot/brot3/commit/9654096b392b160c5f041a58bb71082c979f3ecc))
- Add Display, Debug and basic string parsing for BigVec2 and BigComplex - ([8822d0e](https://github.com/crazyscot/brot3/commit/8822d0e68f2e48e8b9cc910d1f92bbeba55f0539))
- Switch off clippy::missing_panics_doc altogether - ([f65c852](https://github.com/crazyscot/brot3/commit/f65c85213f5bcc4305a3f639a2f55b1605d8d36e))
- Drop now-empty util crate, update ci - ([323fbbb](https://github.com/crazyscot/brot3/commit/323fbbb4b95b939ebf3c87df412cdb5dd9d32626))
- Cargo autoderive, resort the deps lists, update workspace structure diagram - ([4cdbc24](https://github.com/crazyscot/brot3/commit/4cdbc245c8ba16f46b86c298f403c7e6cd315c13))
- Update copyright/authorship notices - ([e47efe9](https://github.com/crazyscot/brot3/commit/e47efe92ef059d6c390332c1b52b7b25c3125f05))
- Reduce visibility of internal lib items - ([ae24700](https://github.com/crazyscot/brot3/commit/ae2470097f365f64b83e7588b4c1d1c020963e44))
- Add a cfg_alias for spirv - ([4fa341a](https://github.com/crazyscot/brot3/commit/4fa341af2295cb2359f4885a73c84bd6e5078a47))
- Add --no-parallel-render flag, to aid analysis - ([fa99235](https://github.com/crazyscot/brot3/commit/fa992352bac7d7980fd44a3d7d94f0c6e0cc0240))
- Cargo autoinherit - ([0d3f530](https://github.com/crazyscot/brot3/commit/0d3f530bd99499489369ff9d4764a74b9d86fc22))
- Impl FromStr for BigComplex - ([ce06fd0](https://github.com/crazyscot/brot3/commit/ce06fd0e58f7716370dec109f6a5d1c87ce82557))

### 💼 Other

- Introduce dprintln macro, use it for save fn chatter - ([4a03f7f](https://github.com/crazyscot/brot3/commit/4a03f7f4f9ac692a4144aebd4dd7b451734d49ad))
- Improve pixel error handling - ([88d406f](https://github.com/crazyscot/brot3/commit/88d406f547fec1125e97345ed683df5f20a9a9c1))
- Remove clippy from existing builds - ([15228d4](https://github.com/crazyscot/brot3/commit/15228d41e0ac861bdcb2f97377b12d3c5bdc142b))

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
