# Texbirds/VIctory — Merged Upstream PR Manifest (
ext)

This branch integrates **153 vetted upstream pull requests** from [Marvisak/VIctory](https://github.com/Marvisak/VIctory) on top of upstream main (63bcca4), preserving every contributor's original Git commit authorship (git merge --no-ff).

All merged changes pass:
- cargo run --locked -p xtask -- policy (strict [asset policy](docs/asset-policy.md) compliance, no game assets, all source files < 512 KiB)
- cargo clippy --workspace --all-targets --locked -- -D warnings (0 warnings)
- cargo build --release --locked
- Workspace unit tests (ictory-core, ictory-formats, ictory-pe, ictory-loader, ictory-game, ictory-hle, ictory-pipeline) and hardware/WARP GPU & shader tests (ictory-gpu, ictory-shader --features host).

## Merged Pull Requests (153 total)

| PR | Title | Author |
|----|-------|--------|
| [#3097](https://github.com/Marvisak/VIctory/pull/3097) | Shaders: lower's tests in their own file, under the policy's size limit | neonbyte1 |
| [#2628](https://github.com/Marvisak/VIctory/pull/2628) | Shaders: on a capped heap, title indices past the title's slots read the null view | moe-miau |
| [#2626](https://github.com/Marvisak/VIctory/pull/2626) | Check packed texture copies on a nonzero mip | andrewpozdnakov7-jpg |
| [#2101](https://github.com/Marvisak/VIctory/pull/2101) | GPU: castable formats the host has | Deilach |
| [#2118](https://github.com/Marvisak/VIctory/pull/2118) | GPU: the held BVH builder dispatch warning names both reasons | 0xkn0wh3r3 |
| [#2703](https://github.com/Marvisak/VIctory/pull/2703) | GPU: a root constant's dword 4 is a value, not a view offset (#2699) | Calvix2k |
| [#2596](https://github.com/Marvisak/VIctory/pull/2596) | Shader tests: an empty root-constant list is set as the one constant it declares (#2571) | Sebastian Kondratowicz |
| [#2668](https://github.com/Marvisak/VIctory/pull/2668) | Guard heap-direct texture read marks with EXEC | fuzeofficial |
| [#3038](https://github.com/Marvisak/VIctory/pull/3038) | Shaders: keep spilled compute root parameters from claiming s16+ system SGPRs | fuzeofficial |
| [#3043](https://github.com/Marvisak/VIctory/pull/3043) | Shaders, GPU: clamp immediate static sampler MipLODBias to D3D12 bounds | fuzeofficial |
| [#1129](https://github.com/Marvisak/VIctory/pull/1129) | GPU: pipeline workers get a 64 MB stack for the driver's compiler | Peppacodes |
| [#2309](https://github.com/Marvisak/VIctory/pull/2309) | Shaders: wave64 on 32-lane hosts + one back edge per block | desp |
| [#1473](https://github.com/Marvisak/VIctory/pull/1473) | Shaders: track per-lane export state across mixed pixel-shader joins | Tex |
| [#3047](https://github.com/Marvisak/VIctory/pull/3047) | Game: name rasterisers from the pack's register images | zed6000 |
| [#3048](https://github.com/Marvisak/VIctory/pull/3048) | Game: try what depth-stencil registers do not keep | zed6000 |
| [#3016](https://github.com/Marvisak/VIctory/pull/3016) | Shaders: fold the waterfall loop's SCC only for the keyed form, under a loop limit it cannot reach | Anzymanz |
| [#2363](https://github.com/Marvisak/VIctory/pull/2363) | GPU: descriptor dwords a shader reads as data - the record count and the format | Calvix2k |
| [#2978](https://github.com/Marvisak/VIctory/pull/2978) | Check actual adapter support in the Wave32 tests | andrewpozdnakov7-jpg |
| [#3083](https://github.com/Marvisak/VIctory/pull/3083) | Shaders/Decode: always decode GFX10 v_cmpx_* VOP3 and SDWA destination as EXEC | RVZO6 |
| [#3056](https://github.com/Marvisak/VIctory/pull/3056) | Shaders/Lower: guard global_atomic_fmin and global_atomic_fmax against NaN operands | RVZO6 |
| [#3379](https://github.com/Marvisak/VIctory/pull/3379) | GPU: deduplicate HostList::keep refs, stack-allocate user_data, and avoid double CBV shadow lookup | Tex |
| [#3312](https://github.com/Marvisak/VIctory/pull/3312) | GPU: marked ranges of copies in video memory are not read back whole | neonbyte1 |
| [#2085](https://github.com/Marvisak/VIctory/pull/2085) | Shader tests: the depth-test draw fills its UAV before drawing | 0xkn0wh3r3 |
| [#2914](https://github.com/Marvisak/VIctory/pull/2914) | GPU: marked pages go back to title memory on several threads | Machad3x |
| [#3035](https://github.com/Marvisak/VIctory/pull/3035) | GPU: write faults read the bytes they save without the guard lock | liandraraiane |
| [#3484](https://github.com/Marvisak/VIctory/pull/3484) | GPU: tidy bindless texture staging after review | desamodeth |
| [#3046](https://github.com/Marvisak/VIctory/pull/3046) | Run, GPU: save the pipeline cache on normal title exit | fuzeofficial |
| [#3096](https://github.com/Marvisak/VIctory/pull/3096) | GPU: cut CPU work when preparing submissions | Mozze0 |
| [#3052](https://github.com/Marvisak/VIctory/pull/3052) | GPU: read-mark scans stop at the end of the view slots in use | 0xkn0wh3r3 |
| [#3005](https://github.com/Marvisak/VIctory/pull/3005) | GPU: make already-used pipeline lookups read-only | Aws |
| [#3018](https://github.com/Marvisak/VIctory/pull/3018) | HLE: make the dstorage share test exercise delete sharing | Halfect |
| [#1543](https://github.com/Marvisak/VIctory/pull/1543) | GPU: avoid shift and arithmetic overflow panics in subresource and copyable_footprints | Tex |
| [#2444](https://github.com/Marvisak/VIctory/pull/2444) | HLE: keep refilling XMA PCM buffer across zero-sample decode steps | Tex |
| [#2445](https://github.com/Marvisak/VIctory/pull/2445) | Loader: clamp VEH instruction reads to the faulting page in emulate_amd_only and sse4a | Tex |
| [#3111](https://github.com/Marvisak/VIctory/pull/3111) | Formats/BCPack: restrict BC7 channel rotation to modes 4 and 5 | RVZO6 |
| [#3084](https://github.com/Marvisak/VIctory/pull/3084) | Wait for queued presents before resizing the swap chain | Yaohui Xie |
| [#3057](https://github.com/Marvisak/VIctory/pull/3057) | GPU/Resource: support 2-plane D16_UNORM_S8_UINT formats (118..=120) in GetCopyableFootprints | RVZO6 |
| [#3041](https://github.com/Marvisak/VIctory/pull/3041) | GPU: refresh heap textures with untracked pages (FIND-17) | leftmove |
| [#2896](https://github.com/Marvisak/VIctory/pull/2896) | GPU: a list that binds no view heap gets ours before the root signature | BastienRdz |
| [#2474](https://github.com/Marvisak/VIctory/pull/2474) | GPU: refreshes of two parts of one page do not make it current whole | BastienRdz |
| [#3377](https://github.com/Marvisak/VIctory/pull/3377) | Shaders: lower partial-channel typed and indexed sub-dword buffer stores | Tex |
| [#3403](https://github.com/Marvisak/VIctory/pull/3403) | Shaders: lower partial-channel image_store and key texture UAV handles by view_kind | Tex |
| [#2398](https://github.com/Marvisak/VIctory/pull/2398) | Shaders, GPU: prune linked mesh/pixel signatures under the 32 KiB DXIL budget | Tex |
| [#1540](https://github.com/Marvisak/VIctory/pull/1540) | Shaders: guard dual loop back-edge lowering | Tex |
| [#2312](https://github.com/Marvisak/VIctory/pull/2312) | GPU, Shaders: spread compute dispatches past 65,535 groups in x across z | andrewpozdnakov7-jpg |
| [#3197](https://github.com/Marvisak/VIctory/pull/3197) | GPU: batch indirect argument clamps into a single dispatch | Sleither Reyes |
| [#2958](https://github.com/Marvisak/VIctory/pull/2958) | Shaders: coalesce per-lane dirty-page marks at wave level | neonbyte1 |
| [#2122](https://github.com/Marvisak/VIctory/pull/2122) | Shaders: fix RDNA4 buffer size handle | imjustprism |
| [#2787](https://github.com/Marvisak/VIctory/pull/2787) | CLI/GPU: avoid rescanning game_dir and re-inflating/re-hashing cached containers during warm prebuild, and stop Named::find early | RVZO6 |
| [#1722](https://github.com/Marvisak/VIctory/pull/1722) | GPU: the host pipeline library works under Wine (null blob for an empty library) | Impre-visible |
| [#2254](https://github.com/Marvisak/VIctory/pull/2254) | Shaders: v_mad_i64_i32 sign-extends a 32-bit literal addend | BastienRdz |
| [#2917](https://github.com/Marvisak/VIctory/pull/2917) | Shaders: round half add and subtract toward zero once | andrewpozdnakov7-jpg |
| [#2877](https://github.com/Marvisak/VIctory/pull/2877) | GPU: make frame pacing tests deterministic with a virtual clock | alexandruiulian201-bot |
| [#188](https://github.com/Marvisak/VIctory/pull/188) | Loader: find TLS-array load candidates sixteen offsets at a time (8-9x faster scan) | TG0000 |
| [#2065](https://github.com/Marvisak/VIctory/pull/2065) | HLE: allow oleaut32.dll in default host policy | fuzeofficial |
| [#2706](https://github.com/Marvisak/VIctory/pull/2706) | victory-gpu: keep a pipeline library per translation, not one shared one (a translator edit destroyed a 926 MB / 7.5 min cache) | Nazar Ukolov |
| [#3070](https://github.com/Marvisak/VIctory/pull/3070) | HLE net: classify bracketed IPv6 literals as IP literals in reachable_name | Dukestar69 |
| [#2026](https://github.com/Marvisak/VIctory/pull/2026) | Formats: bounds-check a detiled row once, not each run in it | 0xkn0wh3r3 |
| [#491](https://github.com/Marvisak/VIctory/pull/491) | GPU: remember pipelines whose creation crashed the process (VICTORY_PIPELINE_JOURNAL) | desp |
| [#2230](https://github.com/Marvisak/VIctory/pull/2230) | GameInput: a tap between two current-state reads is not lost (input at low frame rates) | desp |
| [#2528](https://github.com/Marvisak/VIctory/pull/2528) | GPU diagnostics: count texture data without row padding | andrewpozdnakov7-jpg |
| [#2022](https://github.com/Marvisak/VIctory/pull/2022) | xmem: an aligned XMemAllocDefault block is padded once, not twice | 0xkn0wh3r3 |
| [#2184](https://github.com/Marvisak/VIctory/pull/2184) | xmem: 32-bit XMemAllocDefault blocks get the alignment they ask for | BastienRdz |
| [#1975](https://github.com/Marvisak/VIctory/pull/1975) | HLE: match rpcrt4 UuidFromString nil-UUID and canonical string rules | mgunes1311-lab |
| [#1732](https://github.com/Marvisak/VIctory/pull/1732) | GPU: f32_to_half keeps NaN and rounds normals to nearest even (#843) | andrewpozdnakov7-jpg |
| [#2657](https://github.com/Marvisak/VIctory/pull/2657) | Shaders: scope DXIL bitcode liveness to reachable blocks and live phi edges | Bartosz |
| [#1434](https://github.com/Marvisak/VIctory/pull/1434) | Loader: match title DLL filenames ignoring ASCII case | fuzeofficial |
| [#2141](https://github.com/Marvisak/VIctory/pull/2141) | HLE: registry lookups through kernelbase.dll reach the device-type overrides | BastienRdz |
| [#2195](https://github.com/Marvisak/VIctory/pull/2195) | GPU tests: the library test's shaders compile with Wine's compiler too | Winfried Gerlach |
| [#2249](https://github.com/Marvisak/VIctory/pull/2249) | HLE: WSAConnectByList and GetAddrInfoExA fail where the host lacks them, not abort (Wine) | slaktarn |
| [#2658](https://github.com/Marvisak/VIctory/pull/2658) | Game tests: cover CityHash64 length and block boundaries | XeNS1 |
| [#2450](https://github.com/Marvisak/VIctory/pull/2450) | Shader: check device removal before FMad readback | Damokles2 |
| [#2480](https://github.com/Marvisak/VIctory/pull/2480) | Shaders: BDD meet rejects a node held both ways and takes one condition as met | liandraraiane |
| [#2664](https://github.com/Marvisak/VIctory/pull/2664) | Shaders/IR: eliminate per-instruction/CFG Vec allocations and O(1) effects dedup | RVZO6 |
| [#1701](https://github.com/Marvisak/VIctory/pull/1701) | GPU: VICTORY_MODE overrides the reported console display mode | Calvix2k |
| [#2686](https://github.com/Marvisak/VIctory/pull/2686) | Shaders: validate far-jump arithmetic and control-flow entries | XeNS1 |
| [#2694](https://github.com/Marvisak/VIctory/pull/2694) | Reject XMA frame starts outside the announcing packet | Codex |
| [#2710](https://github.com/Marvisak/VIctory/pull/2710) | Apply outstanding XMA sample skips to the final overlap | Codex |
| [#2714](https://github.com/Marvisak/VIctory/pull/2714) | Keep XMA input valid when looping back to the start of a buffer | Codex |
| [#2808](https://github.com/Marvisak/VIctory/pull/2808) | Shaders: speed up DXBC container MD5 hashing by 16-43% | brainbudt |
| [#2690](https://github.com/Marvisak/VIctory/pull/2690) | GPU: report default buffer alignment from GetDesc | andrewpozdnakov7-jpg |
| [#2846](https://github.com/Marvisak/VIctory/pull/2846) | docs: fix the dangling HDR TODO reference | Codex |
| [#2524](https://github.com/Marvisak/VIctory/pull/2524) | HLE: bounds-check all four bytes of ACP notification writes in audio::submit | shh |
| [#1428](https://github.com/Marvisak/VIctory/pull/1428) | GPU: guard OMSetRenderTargets handle expansion on empty, null, or oversized calls | fuzeofficial |
| [#2823](https://github.com/Marvisak/VIctory/pull/2823) | Shaders: the neg input modifier flips a NaN's sign too | KineticK |
| [#2738](https://github.com/Marvisak/VIctory/pull/2738) | Tests: wait for the late page list instead of sleeping | Therecalloftheonion |
| [#2741](https://github.com/Marvisak/VIctory/pull/2741) | PE: reject null IAT RVAs in non-empty import descriptors | fuzeofficial |
| [#2511](https://github.com/Marvisak/VIctory/pull/2511) | XGameSave: refuse an update when no version number is above the highest (#2497) | Sebastian Kondratowicz |
| [#2749](https://github.com/Marvisak/VIctory/pull/2749) | XTaskQueue: every queue using a port hears of a task submitted to it | F1reworks |
| [#379](https://github.com/Marvisak/VIctory/pull/379) | Shaders: lower v_div_fixup_f16 | Luiz Neves |
| [#1877](https://github.com/Marvisak/VIctory/pull/1877) | Shaders: hardware test for the 16-bit integer ops | Luiz Neves |
| [#2516](https://github.com/Marvisak/VIctory/pull/2516) | GPU capture: decode XR-biased RGB before image scaling | andrewpozdnakov7-jpg |
| [#2784](https://github.com/Marvisak/VIctory/pull/2784) | Shaders: build Halves def/phi table lazily as a compact Vec<Def> in io::raise | RVZO6 |
| [#541](https://github.com/Marvisak/VIctory/pull/541) | GPU: retain D3D12 object names in private data | broccolize |
| [#2520](https://github.com/Marvisak/VIctory/pull/2520) | Shaders: walk a predecessor's live-out registers once instead of searching it per register | liandraraiane |
| [#2789](https://github.com/Marvisak/VIctory/pull/2789) | Shaders: ds_wrap_rtn_b32 reads both of its data operands | lcg007xd-c |
| [#2605](https://github.com/Marvisak/VIctory/pull/2605) | GPU: waits on memory that cannot be read are counted apart (#2604) | BastienRdz |
| [#2793](https://github.com/Marvisak/VIctory/pull/2793) | Shaders: signature rows of one register follow their name offsets | Devin |
| [#2798](https://github.com/Marvisak/VIctory/pull/2798) | Shaders: a mesh program's PSV0 counts shared memory only when it has some | Devin |
| [#2567](https://github.com/Marvisak/VIctory/pull/2567) | Diagnostics: capture RGBA32 unsigned targets | andrewpozdnakov7-jpg |
| [#1802](https://github.com/Marvisak/VIctory/pull/1802) | Perf/shader relative shift bounds | VIREX |
| [#1849](https://github.com/Marvisak/VIctory/pull/1849) | GPU: free memory is not the runtime's to watch (fixes a world-load deadlock under Wine) | crashbashash |
| [#1857](https://github.com/Marvisak/VIctory/pull/1857) | Shader tests: 64-bit image atomics skip devices without typed 64-bit heap atomics | Ingenioeren |
| [#1907](https://github.com/Marvisak/VIctory/pull/1907) | victory-shader: honor unwritten OSG1 channels and output modes in rectangle_gs | Mehmet G├╝ne┼ƒ |
| [#1920](https://github.com/Marvisak/VIctory/pull/1920) | Shaders: v_mad_f32 and v_mac_f32 multiply apart from their add | kais7654 |
| [#1927](https://github.com/Marvisak/VIctory/pull/1927) | Shaders: d16 format loads saturate 32-bit integer channels | upipibi |
| [#1933](https://github.com/Marvisak/VIctory/pull/1933) | victory-gpu: keep depth-twin UAVs in concrete UAV formats and support default views | Mehmet G├╝ne┼ƒ |
| [#1944](https://github.com/Marvisak/VIctory/pull/1944) | Shaders: reject a dxbc part inside the header or part table | imjustprism |
| [#1244](https://github.com/Marvisak/VIctory/pull/1244) | Validate VICTORY_PAIRS before translating shaders | DoubleD17 |
| [#1978](https://github.com/Marvisak/VIctory/pull/1978) | GPU: no texture layouts or name copies per draw for unchanged answers | 0xkn0wh3r3 |
| [#2005](https://github.com/Marvisak/VIctory/pull/2005) | GPU: log the missing D3D12 1.7 (no castable texture formats) | Calvix2k |
| [#2512](https://github.com/Marvisak/VIctory/pull/2512) | GameInput: look up only devices already seen by the process | andrewpozdnakov7-jpg |
| [#2020](https://github.com/Marvisak/VIctory/pull/2020) | Shaders: abs / neg on the shift-add and align ops act on the sign bit | upipibi |
| [#1995](https://github.com/Marvisak/VIctory/pull/1995) | CLI: recommend a DXIL validator in translate --help | AlanQiang0418 |
| [#2293](https://github.com/Marvisak/VIctory/pull/2293) | GPU: a slot rewritten during a run keeps its view's copy of title memory until the run is over | denkencar-sketch |
| [#2095](https://github.com/Marvisak/VIctory/pull/2095) | Input: trigger rumble through Windows.Gaming.Input | Will Bartlett |
| [#2504](https://github.com/Marvisak/VIctory/pull/2504) | HLE: physical-page writes open the title's views before they write | BastienRdz |
| [#2299](https://github.com/Marvisak/VIctory/pull/2299) | GPU: harden CopyMemoryToMemoryX against unmapped or read-only pages | denkencar-sketch |
| [#2303](https://github.com/Marvisak/VIctory/pull/2303) | GPU: page mapping copies wait for the submission that is running | denkencar-sketch |
| [#3104](https://github.com/Marvisak/VIctory/pull/3104) | HLE XGameSave: tolerate locked old or unfinished version cleanup | fuzeofficial |
| [#1361](https://github.com/Marvisak/VIctory/pull/1361) | Shaders: hardware probe for s_flbit_i32 | Luiz Neves |
| [#2623](https://github.com/Marvisak/VIctory/pull/2623) | Shaders: round half-precision fused multiply-add once | andrewpozdnakov7-jpg |
| [#2585](https://github.com/Marvisak/VIctory/pull/2585) | GPU tests: each test's prepare hook runs on its own thread | fyernald |
| [#2564](https://github.com/Marvisak/VIctory/pull/2564) | Shaders: hardware probes for the remaining s_cmp_* integer compares | Luiz Neves |
| [#2565](https://github.com/Marvisak/VIctory/pull/2565) | Shaders: hardware probes for s_bitcmp0_b32 and s_bitcmp1_b32 | Luiz Neves |
| [#2570](https://github.com/Marvisak/VIctory/pull/2570) | Shaders: hardware probes for the 64-bit scalar logic and shift ops | Luiz Neves |
| [#2559](https://github.com/Marvisak/VIctory/pull/2559) | GPU: a removal report without DRED names the switch, not 'unsupported' | Ingenioeren |
| [#2486](https://github.com/Marvisak/VIctory/pull/2486) | GPU: keep constant nonzero targets visible in captures | andrewpozdnakov7-jpg |
| [#2072](https://github.com/Marvisak/VIctory/pull/2072) | HLE: an unknown gamertag component is an invalid argument | 0xkn0wh3r3 |
| [#2491](https://github.com/Marvisak/VIctory/pull/2491) | Shaders: intersect compare bounds with existing range bounds in Ranges::edge_hi | shh |
| [#3092](https://github.com/Marvisak/VIctory/pull/3092) | Loader: keep concurrent crash records whole | neonbyte1 |
| [#2238](https://github.com/Marvisak/VIctory/pull/2238) | Watch: dropping a tracker keeps a mapping guarded while a tracker holds its alias | denkencar-sketch |
| [#3049](https://github.com/Marvisak/VIctory/pull/3049) | Game: recover SGD5 records from unparsed databases alongside intact ones | fuzeofficial |
| [#1717](https://github.com/Marvisak/VIctory/pull/1717) | HLE: the retail console ID has the documented dotted form | davutsimsekk |
| [#2276](https://github.com/Marvisak/VIctory/pull/2276) | GPU: a depth texture used before it got a twin is copied to the twin in the list | denkencar-sketch |
| [#1866](https://github.com/Marvisak/VIctory/pull/1866) | Game: preserve sample masks in prebuilt graphics pipelines | necrobytes |
| [#2284](https://github.com/Marvisak/VIctory/pull/2284) | Shader tests: cover the merged geometry user-data layout | Halfect |
| [#2075](https://github.com/Marvisak/VIctory/pull/2075) | HLE: with no process task queue, a call given no queue fails with E_NO_TASK_QUEUE | 0xkn0wh3r3 |
| [#3044](https://github.com/Marvisak/VIctory/pull/3044) | CLI: refuse VICTORY_PREBUILD_KEYS paths inside the repository | fuzeofficial |
| [#1819](https://github.com/Marvisak/VIctory/pull/1819) | Shaders: translated loops are left after 2^14 iterations (was 2^20) | Exopus |
| [#2090](https://github.com/Marvisak/VIctory/pull/2090) | Add CPU-only execution-equivalence tests for Wave32 shader SSA and structured EXEC | Chase Brannen |
| [#2501](https://github.com/Marvisak/VIctory/pull/2501) | Shaders: a BVH node offset from a root descriptor is computed in 64 bits | Calvix2k |
| [#2542](https://github.com/Marvisak/VIctory/pull/2542) | GPU: use non-zero NumElements for null raw pairs in HostHeap::raw_pair_at | shh |
| [#1796](https://github.com/Marvisak/VIctory/pull/1796) | Core: report unreadable implicit configuration files | VIREX |
| [#1823](https://github.com/Marvisak/VIctory/pull/1823) | Shaders: bound a signature part's register rows and its allocation | slatefield |
| [#3020](https://github.com/Marvisak/VIctory/pull/3020) | CLI: count all SGD5 hashes and stripped-bitcode XDAT/xbop markers in shader-info | fuzeofficial |
| [#1981](https://github.com/Marvisak/VIctory/pull/1981) | Preserve the last complete save when a newer version is missing a blob | WerG0D |
| [#2981](https://github.com/Marvisak/VIctory/pull/2981) | HLE/DirectStorage: reuse per-worker DecompressorOxide and source/intermediate scratch buffers | RVZO6 |
| [#2589](https://github.com/Marvisak/VIctory/pull/2589) | Diagnostics: preserve the world capture draw count | andrewpozdnakov7-jpg |
| [#2646](https://github.com/Marvisak/VIctory/pull/2646) | GPU: a WARP host keeps its pipeline library in a file of its own (#2637) | andrewpozdnakov7-jpg |
| [#2702](https://github.com/Marvisak/VIctory/pull/2702) | GPU: resolve multisampled color targets before diagnostic readback | andrewpozdnakov7-jpg |
| [#2904](https://github.com/Marvisak/VIctory/pull/2904) | DirectStorage: workers read a file through a handle each | 0xkn0wh3r3 |
| [#2902](https://github.com/Marvisak/VIctory/pull/2902) | HLE: a module name with a trailing dot gets no .dll appended | pawrenity |
