// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	declare module '*.svg?component' {
		import type { ComponentType, SvelteComponentTyped } from 'svelte';
		import type { SVGAttributes } from 'svelte/elements';

		const content: ComponentType<SvelteComponentTyped<SVGAttributes<SVGSVGElement>>>;

		export default content;
	}

	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
