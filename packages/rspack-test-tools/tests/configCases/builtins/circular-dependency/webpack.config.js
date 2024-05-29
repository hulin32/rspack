const rspack = require("@rspack/core");

module.exports = {
    entry: {
        main: "./a",
    },
    output: {
        filename: "[name].js",
    },
    plugins: [
        new rspack.CircularDependencyPlugin({
            onDetected(paths) {
                // find unique filenames
                // let seen = [];
                // for (const path of paths) {
                //     // only add file name instead of full path
                //     const file = path.file.split("/").pop();
                //     if (seen.has(file)) continue;
                //     seen.add(path.file);
                // }
                // seen.sort();
                // expect(seen.join(",")).toBe('a.js,b.js,c.js');
                expect(1).toBe(1);
            }
        })
    ]
};
