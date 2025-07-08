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
		...props
	}: Props = $props();
</script>

<input
	type="text"
	bind:value
	autocomplete="off"
	onkeydown={(evt) => {
		if (evt.key === 'Enter') {
			onsubmit?.(value);
		}
	}}
	{...props}
	onchange={() => onchange?.(value)}
	class={[
		classProp,
		`text-${size}`,
		`placeholder:text-${size}`,
		'valid:focus:ring-accent-500 disabled:text-primary-400 bg-primary-900 text-primary-300 placeholder-primary-400 enabled:hover:ring-primary-500 min-w-0 grow rounded-lg px-3 py-1 invalid:ring-1 invalid:ring-red-500 invalid:hover:ring-2 invalid:hover:ring-red-500 focus:ring-2 focus:outline-hidden enabled:hover:ring-1 disabled:cursor-not-allowed'
	]}
/>
