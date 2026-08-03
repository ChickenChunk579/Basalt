function buildBar(b) {
	const clang = b.toolchains.clang(b);
    const raylib = b.dependency("raylib");

	const bar = b.library("bar", {
		sources: b.glob("src/**/*.c"),
		dependencies: [raylib],
		toolchain: clang,
		isStatic: true,
		includeDirectories: ["src/"],
    	define: []
	});

	return bar;
}

function build(b) {
	return [buildBar(b)];
}

function pkg(b) {
	return {
		bar: buildBar(b)
	}
}

