const path = __non_webpack_require__("path");

it("css modules composes", () => {
	const style = require("./index.css");
	expect(style).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'index.css.txt'));
});
