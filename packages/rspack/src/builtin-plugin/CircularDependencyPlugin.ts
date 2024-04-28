import {
	RawCircularDependencyPluginOptions,
	BuiltinPluginName
} from "@rspack/binding";
import { create } from "./base";

export const CircularDependencyPlugin = create(
	BuiltinPluginName.CircularDependencyPlugin,
	(options: RawCircularDependencyPluginOptions) => options
);

export type CircularDependencyPluginOptions =
	RawCircularDependencyPluginOptions;
