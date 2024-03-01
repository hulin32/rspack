const path = require("path");

/** @type {import('jest').Config} */
const config = {
	testEnvironment: "../../scripts/test/patch-node-env.cjs",
	setupFilesAfterEnv: ["<rootDir>/tests/setupTestFramework.js"],
	testMatch: [
		"<rootDir>/tests/*.test.js",
		"<rootDir>/tests/*.basictest.js",
		"<rootDir>/tests/*.longtest.js",
		"<rootDir>/tests/*.unittest.js",
		"<rootDir>/tests/copyPlugin/*.test.js"
	],
	testTimeout: process.env.CI ? 60000 : 30000,
	prettierPath: require.resolve("prettier-2"),
	moduleNameMapper: {
		// Fixed jest-serialize-path not working when non-ascii code contains.
		slash: path.join(__dirname, "../../scripts/test/slash.cjs")
	},
	cache: false,
	transformIgnorePatterns: ["<rootDir>/tests"],
	snapshotFormat: {
		escapeString: true,
		printBasicPrototype: true
	}
};

module.exports = config;
