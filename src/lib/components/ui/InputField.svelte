<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	type Props = {
		value?: string;
		size?: 'sm' | 'md' | 'lg';
		class?: string;
		onsubmit?: (value: string) => void;
		onchange?: (value: string) => void;
	} & Omit<HTMLInputAttributes, 'size' | 'onchange'>;

	let {
		value = $bindable(''),
		size = 'md',
		class: classProp = '',
		onsubmit,
		onchange,
		spellcheck = false,
		autocomplete = 'off',
		...props
	}: Props = $props();
</script>

<input
	type="text"
	bind:value
	onkeydown={(evt) => {
		if (evt.key === 'Enter') {
			onsubmit?.(value);
		}
	}}
	{autocomplete}
	{spellcheck}
	{...props}
	onchange={() => onchange?.(value)}
	class={[
		classProp,
		`text-${size}`,
		`placeholder:text-${size}`,
		'valid:focus:ring-accent-600! text-primary-700 placeholder-primary-500 enabled:hover:ring-primary-400 dark:valid:focus:ring-accent-500! dark:bg-primary-900 dark:text-primary-300 dark:placeholder-primary-400 dark:enabled:hover:ring-primary-500 bg-primary-100 min-w-0 grow rounded-lg px-3 py-1 invalid:ring-1 invalid:ring-red-500 invalid:hover:ring-2 invalid:hover:ring-red-500! focus:ring-2! focus:outline-hidden enabled:hover:ring-1 disabled:cursor-not-allowed disabled:opacity-70'
	]}
/>
