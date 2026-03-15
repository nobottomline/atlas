Atlas

A next-generation on-device source platform for iOS manga applications

Tagline: A professional, secure, open-source source ecosystem for manga providers — fully on-device, plugin-driven, and architected for long-term scale.

⸻

1. Executive Summary

Atlas is a source-platform architecture for an iOS manga reader/downloader application.

The goal is not merely to support “sources” or “extensions,” but to build a first-class source ecosystem that is:
	•	fully on-device,
	•	modular,
	•	open-source friendly,
	•	professionally versioned,
	•	security-conscious,
	•	testable,
	•	scalable,
	•	and architecturally superior to existing community-source systems.

Atlas is designed around a strict separation of concerns:
	1.	Host App — the iOS application built with Swift/SwiftUI and Tuist.
	2.	Source Runtime — a sandboxed execution environment for external source modules.
	3.	Source SDK — a Rust-based developer toolkit for authoring source modules.
	4.	Source Registry — a signed, versioned repository of installable source packages.
	5.	Source Policy & Capability Model — a permission and limitation system controlling what sources can do.
	6.	Conformance / CI / Tooling — a professional-grade validation and release pipeline.

The platform is intended for research, power users, and open-source ecosystem development first, without making App Store compatibility the initial architectural constraint.

The design philosophy is simple:

Do not build “a bunch of parsers.” Build a platform for source modules.

⸻

2. Project Name

Primary Name: Atlas

Why “Atlas”
	•	Strong, memorable, premium-sounding.
	•	Implies a structured map of many worlds/sources.
	•	Feels infrastructural, foundational, and scalable.
	•	Works well as an open-source project name.
	•	Broad enough to cover future expansion beyond manga.

Brand Positioning

Atlas is not just a reader feature. It is the source platform powering a next-generation manga client.

Possible Sub-Brands
	•	Atlas Core — runtime and host integration
	•	Atlas SDK — Rust developer kit for source authors
	•	Atlas Registry — signed source distribution index
	•	Atlas CLI — developer tooling
	•	Atlas Spec — schema and standards repository
	•	Atlas TestKit — conformance and fixture suite

⸻

3. Vision

Product Vision

To create the most professional and extensible on-device manga source architecture for iOS, enabling independent source modules to be authored, distributed, installed, updated, debugged, and maintained without embedding all parsing logic directly into the application.

Strategic Vision

Atlas should become:
	•	a cleaner architectural evolution beyond typical extension ecosystems,
	•	an open-source reference implementation for source-driven content apps on Apple platforms,
	•	a developer-friendly ecosystem that encourages quality, testing, and consistent contracts,
	•	and a future-proof source platform that can later support additional media/content categories if desired.

⸻

4. Core Principles

4.1 Platform, not hack

Atlas must feel like a real platform with standards, contracts, versioning, capabilities, lifecycle management, and tooling.

4.2 On-device first

All source execution happens locally on the user’s device. No required central server. No mandatory gateway.

4.3 Strong isolation

A source must not be treated as trusted application code. It must run within a controlled sandbox with limited capabilities.

4.4 Strict contracts

Source modules must implement clear typed contracts for search, details, chapters, pages, settings, and optional authentication.

4.5 Professional maintainability

The ecosystem must support CI, testing, source health checks, signatures, version compatibility, and deprecation mechanisms.

4.6 Open-source friendliness

Source authoring should be possible without owning the host application codebase.

4.7 Scalability over shortcuts

Avoid fragile, ad-hoc solutions that only work for a few sources. Design for hundreds of sources.

⸻

5. High-Level Architecture

Atlas consists of five major layers.

Layer A — Host Application

The user-facing iOS app.

Responsibilities:
	•	browsing sources,
	•	installing/updating/removing sources,
	•	invoking source functionality,
	•	storing user preferences,
	•	caching content and metadata,
	•	rendering reader UI,
	•	download management,
	•	sandbox/policy enforcement,
	•	source lifecycle management.

Recommended stack:
	•	Swift
	•	SwiftUI
	•	Tuist
	•	structured concurrency with async/await
	•	local storage with SwiftData or Core Data
	•	URLSession-based networking from host side

Layer B — Source Runtime

An embedded execution environment inside the app responsible for running source modules safely.

Responsibilities:
	•	loading source packages,
	•	validating signatures and hashes,
	•	instantiating modules,
	•	bridging host capabilities,
	•	enforcing memory/time/request limits,
	•	marshalling requests and responses,
	•	isolating failures,
	•	producing structured logs and diagnostics.

Layer C — Source SDK

The toolkit used by extension authors.

Responsibilities:
	•	typed APIs for source development,
	•	standard models and contracts,
	•	helpers for HTML parsing and data mapping,
	•	settings schema generation,
	•	compile pipeline to runtime package format,
	•	local test harness,
	•	conformance helpers.

Primary language:
	•	Rust

Layer D — Source Registry

A repository of source packages and metadata.

Responsibilities:
	•	distribution,
	•	indexing,
	•	semantic versions,
	•	hashes,
	•	signatures,
	•	compatibility constraints,
	•	revocation,
	•	deprecation.

Layer E — Tooling / CI / Validation

The operational layer that keeps the ecosystem healthy.

Responsibilities:
	•	validating manifests,
	•	building modules,
	•	running tests,
	•	generating registry index,
	•	publishing artifacts,
	•	maintaining source health status,
	•	compatibility verification.

⸻

6. Final Technical Direction

Chosen Final Direction

Swift host app + Rust Source SDK + WebAssembly module format + signed registry + capability-based runtime + professional validation/tooling.

This is the project’s final maximum-design direction.

Why this direction

Because it provides the strongest overall balance of:
	•	performance,
	•	safety,
	•	portability,
	•	maintainability,
	•	ecosystem scalability,
	•	open-source development ergonomics,
	•	and architectural cleanliness.

⸻

7. Why Rust

Rust is the best source-authoring language for Atlas because it provides:
	•	strong type safety,
	•	memory safety,
	•	zero-cost abstractions,
	•	mature package tooling,
	•	strong reusable library design,
	•	good parsing ecosystem,
	•	good WASM compilation support,
	•	long-term maintainability,
	•	and a professional developer experience for advanced open-source contributors.

Rust also allows Atlas to build:
	•	shared utility crates,
	•	multisource templates,
	•	typed models,
	•	compile-time guarantees,
	•	and conformance-friendly source implementations.

⸻

8. Why WebAssembly

WebAssembly is the best module distribution format for Atlas because it offers:
	•	sandboxable execution,
	•	platform independence,
	•	small portable packages,
	•	deterministic interfaces,
	•	clean host-guest boundaries,
	•	future compatibility for testing tools and desktop tooling,
	•	and separation between app code and source module code.

Atlas should treat WASM not as a gimmick, but as a portable, constrained module format.

Important Positioning

Atlas is not simply “Rust plugins.”
It is a source platform whose module format is WASM.

⸻

9. Why not other approaches

Kotlin/Gradle/Android-style ecosystem

Rejected because it is too tied to Android architecture and does not cleanly fit a native iOS source platform.

JavaScript-first runtime

Rejected as the primary foundation due to weaker discipline, lower type rigor, less predictable performance, and greater long-term maintenance risk.

Lua/Luau-first runtime

Interesting for embedding, but not ideal as the primary language for a large, professional source ecosystem.

Native dynamic libraries

Too dangerous, too platform-sensitive, and too difficult to manage safely.

Custom VM/DSL from day one

Potentially more powerful in the long term, but too expensive and complex as an initial foundation.

⸻

10. Architectural Goals

Atlas must support:
	•	installation of source packages from registries,
	•	multiple registries,
	•	source updates,
	•	safe removal,
	•	source enable/disable,
	•	source-level preferences,
	•	per-source permissions,
	•	source-level caching,
	•	search/details/chapters/pages contracts,
	•	optional login/auth flows,
	•	rate limiting and request quotas,
	•	diagnostics and logs,
	•	source revocation,
	•	compatibility enforcement,
	•	local testing and CI validation,
	•	and future migration/version evolution.

⸻

11. Source Module Model

Each source is a self-contained package that includes:
	•	a compiled WASM module,
	•	a source manifest,
	•	metadata,
	•	icon assets,
	•	optional locale information,
	•	declared capabilities,
	•	optional settings schema,
	•	compatibility and version information,
	•	integrity hashes,
	•	and a digital signature.

Source Package Responsibilities

A source package is responsible for:
	•	site-specific logic,
	•	search parsing,
	•	content detail extraction,
	•	chapter extraction,
	•	page resolution,
	•	optional login/session handling,
	•	preferences interpretation,
	•	and URL resolution.

A source package is not responsible for:
	•	direct storage access,
	•	unrestricted networking,
	•	arbitrary system calls,
	•	user interface rendering,
	•	unrestricted filesystem usage,
	•	or privileged app behavior.

⸻

12. Capability-Based Runtime Model

Atlas should implement a strict capability model.

A source can only perform operations explicitly allowed by the host.

Example capabilities
	•	network.fetch
	•	network.cookies.read
	•	network.cookies.write
	•	html.parse
	•	cache.read
	•	cache.write
	•	preferences.read
	•	preferences.write
	•	auth.session
	•	time.now
	•	crypto.hash
	•	log.debug

Capability Design Rules
	•	Capabilities must be explicit.
	•	Capabilities must be visible in manifest metadata.
	•	Capabilities must be enforceable by the host.
	•	Capability denial must fail predictably.
	•	Future capability additions must be versioned.

This model improves:
	•	security,
	•	debuggability,
	•	reviewability,
	•	and platform maturity.

⸻

13. Runtime Bridge Design

The host app must provide a carefully designed bridge between Swift and WASM.

Bridge responsibilities
	•	module loading,
	•	request invocation,
	•	result decoding,
	•	capability dispatch,
	•	timeout enforcement,
	•	memory quotas,
	•	structured error mapping,
	•	panic/trap isolation,
	•	and logging.

Invocation model

Preferred model:
	•	host invokes named entry points,
	•	parameters are encoded into a stable message format,
	•	source returns typed responses or structured errors.

Suggested message strategy
	•	Human-readable metadata in JSON
	•	Runtime message payloads in MessagePack or another compact structured format

This gives a good balance between readability and efficiency.

⸻

14. Core Source Contract

Every source must implement a stable interface.

Required functions
	•	get_info()
	•	search(query, filters, page)
	•	get_manga_details(id)
	•	get_chapters(id)
	•	get_pages(chapter_id)

Optional functions
	•	login(credentials)
	•	logout()
	•	get_preferences_schema()
	•	resolve_url(url)
	•	get_filters()
	•	get_latest(page)
	•	get_popular(page)
	•	get_updated(page)

Return types must be standardized

Examples:
	•	SourceInfo
	•	SearchResponse
	•	Manga
	•	Chapter
	•	Page
	•	PreferenceSchema
	•	SourceError

All contracts must be formally versioned.

⸻

15. Manifest Design

Each source package needs a manifest.

Manifest responsibilities
	•	identifying the source,
	•	describing compatibility,
	•	declaring permissions/capabilities,
	•	listing package hashes,
	•	providing metadata for UI and install/update logic.

Example manifest fields
	•	id
	•	name
	•	version
	•	lang
	•	base_urls
	•	content_type
	•	supports_nsfw
	•	module_filename
	•	module_sha256
	•	signature
	•	min_runtime_version
	•	min_app_version
	•	capabilities
	•	preferences_schema_version
	•	author
	•	license
	•	tags
	•	deprecated
	•	replaced_by

The manifest must be validated against a strict schema.

⸻

16. Source Registry Design

Atlas Registry is a signed, versioned catalog of source packages.

Registry goals
	•	deterministic source discovery,
	•	safe updates,
	•	trustable metadata,
	•	compatibility checking,
	•	source lifecycle control.

Registry structure

Recommended structure:
	•	/index.json
	•	/sources/<source-id>/manifest.json
	•	/sources/<source-id>/<version>/module.wasm
	•	/sources/<source-id>/icon.png
	•	/sources/<source-id>/changelog.json
	•	/revocations.json
	•	/signatures.json

Registry metadata should include
	•	latest version,
	•	all published versions,
	•	hash list,
	•	signing metadata,
	•	compatibility range,
	•	deprecation state,
	•	optional quality score,
	•	optional source health state.

Multiple registries

Atlas should support multiple registries:
	•	official registry,
	•	community registries,
	•	private registries,
	•	local/dev registries.

⸻

17. Security Model

Atlas is not a generic plugin host. It is a constrained source execution platform.

Security priorities
	1.	Integrity of installed source packages
	2.	Isolation between source modules
	3.	Controlled network behavior
	4.	Predictable permission model
	5.	Source revocation and update safety
	6.	Defensive parsing and error handling

Core mechanisms
	•	hash verification,
	•	digital signatures,
	•	capability declarations,
	•	domain allowlists per source,
	•	request limits,
	•	timeout limits,
	•	memory limits,
	•	panic isolation,
	•	and source revocation support.

Threats to consider
	•	malicious source package,
	•	compromised registry,
	•	broken update,
	•	abuse via excessive requests,
	•	data exfiltration attempts,
	•	infinite loops / runtime hangs,
	•	malicious HTML/parser edge cases.

⸻

18. Domain and Network Policy

Each source should declare its allowed network domains.

The host enforces:
	•	domain allowlists,
	•	redirect limits,
	•	request quotas,
	•	timeout rules,
	•	user-agent policy,
	•	content-type validation,
	•	and cookie/session scoping.

This turns network behavior into a first-class policy layer rather than leaving it as arbitrary plugin logic.

⸻

19. Caching Strategy

Atlas should define host-managed caching rather than leaving caching behavior entirely to source authors.

Cache layers
	•	registry metadata cache
	•	manifest cache
	•	source response cache
	•	image/page metadata cache
	•	optional HTML snapshot cache for debugging

Cache rules
	•	source modules should request cache access through the host
	•	cache keys should be normalized
	•	cache invalidation should be predictable
	•	host should reserve authority over cache size and eviction

⸻

20. Preferences and Settings

Sources may expose user-configurable settings.

Examples:
	•	preferred domain mirror,
	•	data saver mode,
	•	language preference,
	•	adult-content toggle,
	•	image quality option,
	•	login/session settings.

Design direction

Source settings should be schema-driven.

A source exposes a typed preference schema, and the host app renders native UI from it.

This avoids each source inventing ad-hoc settings behavior.

⸻

21. Error Model

Atlas needs a professional structured error model.

Error categories
	•	network error
	•	parsing error
	•	auth error
	•	capability denied
	•	invalid response
	•	compatibility error
	•	timeout error
	•	source trap/runtime failure
	•	source disabled/revoked

Requirements
	•	all errors must be typed
	•	host should map low-level errors into user-facing categories
	•	source logs should help debugging without leaking unnecessary internals

⸻

22. Source Lifecycle

Each source moves through a lifecycle.

Lifecycle states
	•	discovered
	•	installable
	•	installed
	•	enabled
	•	disabled
	•	update available
	•	deprecated
	•	revoked
	•	incompatible
	•	broken

The app should visibly manage and display these states.

⸻

23. Source Quality Model

Atlas should go further than simple installation.

Each source can have quality metadata.

Example quality dimensions
	•	test coverage presence
	•	last successful CI run
	•	current health status
	•	compatibility confidence
	•	source maintenance status
	•	known issues flag
	•	parser fragility rating

This makes the ecosystem feel mature and transparent.

⸻

24. Developer Experience Goals

Atlas must be pleasant to develop for.

Required developer tooling
	•	source template generator
	•	local test runner
	•	fixture recording/replay
	•	HTML inspection helpers
	•	schema validator
	•	registry linter
	•	package signer
	•	build commands
	•	compatibility checker
	•	debug console integration

DX principle

A source author should be able to:
	1.	scaffold a new source,
	2.	run tests locally,
	3.	build a package,
	4.	validate manifest/schema,
	5.	publish to registry,
	6.	debug runtime behavior.

⸻

25. Atlas CLI

Atlas should include a command-line tool.

Proposed name

atlas

Responsibilities
	•	scaffold source project
	•	build source package
	•	run validation
	•	run tests
	•	sign artifacts
	•	inspect manifest
	•	publish to registry
	•	run local dev registry
	•	debug source invocation

Example commands
	•	atlas new source mangadex
	•	atlas build
	•	atlas test
	•	atlas validate
	•	atlas sign
	•	atlas publish
	•	atlas inspect package.wasm

⸻

26. Repository Strategy

Atlas should be split into multiple repositories or clearly separated packages.

Recommended structure

1. atlas-spec

Contains:
	•	source contracts
	•	schemas
	•	versioning rules
	•	manifest format
	•	capability definitions
	•	compatibility rules

2. atlas-sdk

Contains:
	•	Rust source SDK
	•	types
	•	parsing helpers
	•	bridge abstractions
	•	package build tooling

3. atlas-runtime

Contains:
	•	WASM runtime bridge
	•	host integration logic
	•	policy engine
	•	module execution core

4. atlas-registry

Contains:
	•	published source metadata
	•	source packages
	•	signatures
	•	revocations
	•	release catalog

5. atlas-cli

Contains:
	•	scaffolding and operational tooling

6. atlas-testkit

Contains:
	•	conformance fixtures
	•	test harness
	•	snapshot helpers
	•	regression suite

7. Atlas iOS App

Contains:
	•	the user-facing SwiftUI application
	•	Tuist project definitions
	•	install/update source flows
	•	reader and downloader UI
	•	source management screens

⸻

27. Tuist-Based iOS Architecture

The iOS application should be managed using Tuist for scalability and maintainability.

Why Tuist fits Atlas
	•	modular project structure,
	•	reproducible project generation,
	•	cleaner dependency boundaries,
	•	better long-term project organization,
	•	easier separation between app layers,
	•	more professional infra for a growing codebase.

Suggested app modules
	•	App
	•	Core
	•	UI
	•	Reader
	•	Downloads
	•	Library
	•	Sources
	•	SourceRuntimeBridge
	•	SourceRegistry
	•	Settings
	•	SharedModels
	•	Networking
	•	Persistence
	•	Diagnostics

Suggested internal boundaries
	•	Reader UI should not know runtime internals.
	•	Source runtime should be isolated behind a service boundary.
	•	Registry management should be independent from content rendering.
	•	Shared models should remain clean and versioned.

⸻

28. Build and Packaging Strategy

Source build pipeline
	1.	Author writes source in Rust.
	2.	Source builds against atlas-sdk.
	3.	Build target outputs WASM module.
	4.	Packaging step generates validated manifest.
	5.	Hashes are computed.
	6.	Package is signed.
	7.	Package is published to registry.
	8.	CI runs conformance tests.

App install/update pipeline
	1.	User adds or syncs registry.
	2.	Host fetches registry index.
	3.	Host validates signatures and hashes.
	4.	User installs source.
	5.	Host stores module locally.
	6.	Runtime loads module on demand.
	7.	Updates are detected via registry metadata.
	8.	Host performs verified source replacement.

⸻

29. Conformance and Testing

This is one of the most important parts of the entire project.

Atlas must have a serious conformance suite.

Required testing layers
	•	schema validation
	•	unit tests for source code
	•	fixture-based parser tests
	•	snapshot tests for normalized output
	•	compatibility tests against runtime versions
	•	smoke tests for source contracts
	•	installation/update validation tests
	•	revocation/deprecation behavior tests

Conformance output

Each source should be able to produce a conformance status:
	•	passed
	•	warning
	•	failed

This is what separates a professional platform from a hobby plugin system.

⸻

30. Observability and Diagnostics

Atlas should include source diagnostics.

Recommended diagnostics features
	•	source invocation logs
	•	request trace logs
	•	parsing failure context
	•	capability usage trace
	•	source version/runtime version display
	•	install/update history
	•	structured runtime error reports

Developer mode

A developer mode in the app should allow:
	•	viewing source logs,
	•	inspecting manifest metadata,
	•	testing source entry points,
	•	and checking runtime policy decisions.

⸻

31. Multisource Templates

Atlas should eventually support multisource abstractions.

This means several sites with similar structure can share common source logic.

Benefits
	•	less duplication,
	•	easier maintenance,
	•	faster onboarding of new sources,
	•	better ecosystem consistency.

Design rule

Multisource templates should be built as shared Rust crates and macros, not as copy-paste source folders.

⸻

32. Open-Source Strategy

Atlas should be open-source friendly from the start.

OSS principles
	•	clear contribution rules
	•	stable source contracts
	•	template-based onboarding
	•	transparent CI expectations
	•	versioned releases
	•	source code review standards
	•	signed release pipeline

Community goals

Atlas should attract contributors who care about:
	•	quality,
	•	maintainability,
	•	architecture,
	•	and clean source authoring.

⸻

33. Future Expansion Paths

Although Atlas begins as a manga source platform, the architecture may later extend to:
	•	comics,
	•	novels,
	•	webtoons,
	•	metadata providers,
	•	cover providers,
	•	content sync providers,
	•	and optional download resolvers.

The core architecture should remain generic enough to allow future growth without diluting the current focus.

⸻

34. What makes Atlas better than a basic Aidoku-like system

Atlas improves on older/community source ecosystems by emphasizing:
	•	stronger runtime isolation,
	•	explicit capabilities,
	•	signed registry architecture,
	•	stricter version contracts,
	•	professional conformance tooling,
	•	source health/status metadata,
	•	better diagnostics,
	•	more disciplined packaging,
	•	and a clearer platform mindset.

The goal is not merely to imitate a source ecosystem.
The goal is to professionalize and modernize it.

⸻

35. Phased Implementation Plan

Phase 1 — Foundations
	•	define Atlas Spec
	•	design manifest schema
	•	define source contracts
	•	create Rust SDK skeleton
	•	build minimal WASM runtime bridge
	•	create basic registry format
	•	install a test source end-to-end

Phase 2 — Core Runtime
	•	support search/details/chapters/pages
	•	add capability enforcement
	•	add package validation and signatures
	•	add source install/update/remove flow
	•	add settings schema support
	•	add structured logging

Phase 3 — Tooling
	•	release Atlas CLI
	•	add template scaffolding
	•	add local test harness
	•	add fixture-based parser tests
	•	add registry linter and signer

Phase 4 — Ecosystem Hardening
	•	conformance suite
	•	source health metadata
	•	revocation mechanism
	•	compatibility matrix
	•	multisource abstractions
	•	developer debug tools in app

Phase 5 — Open-Source Expansion
	•	public docs
	•	contributor guides
	•	community registry workflow
	•	formal release process
	•	long-term SDK/runtime versioning policy

⸻

36. Non-Goals

Atlas should explicitly avoid the following as initial goals:
	•	designing a custom VM from scratch immediately,
	•	supporting arbitrary unbounded source scripting languages,
	•	mixing source code directly into the host app repository,
	•	skipping signatures and validation “for now,”
	•	letting sources perform unrestricted networking or storage access,
	•	or overfitting architecture to App Store constraints before the platform exists.

⸻

37. Final Recommendation

Final project recommendation

Build Atlas as a professional source platform with the following final architecture:
	•	Host App: Swift + SwiftUI + Tuist
	•	Source Language: Rust
	•	Module Format: WebAssembly
	•	Runtime Model: capability-based embedded sandbox
	•	Registry: signed, versioned, multi-registry capable
	•	Contracts: strict typed source interfaces
	•	Tooling: CLI + validation + conformance + CI
	•	Execution: fully on-device
	•	Philosophy: platform-first, ecosystem-ready, open-source friendly

Final strategic statement

Atlas should not be treated as a feature.
It should be built as a standalone technical platform that powers manga sources professionally and can outgrow the limitations of older extension ecosystems.

⸻

38. One-Sentence Summary

Atlas is a next-generation on-device source platform for iOS manga apps, built with a Swift/Tuist host, Rust-authored WebAssembly source modules, a signed registry, and a capability-based runtime designed for security, scalability, and open-source ecosystem growth.
