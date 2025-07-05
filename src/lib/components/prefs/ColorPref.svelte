<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import {
		defaultColors,
		getColor,
		setColor,
		type DefaultColor,
		type ColorCategory,
		type Color
	} from '$lib/theme';
	import { capitalize, selectItems } from '$lib/util';
	import Select from '$lib/components/ui/Select.svelte';
	import Icon from '@iconify/svelte';
	import ResetButton from '../ui/ResetButton.svelte';
	import type { ClassValue } from 'clsx';
	import Info from '../ui/Info.svelte';
	import type { Snippet } from 'svelte';
	import clsx from 'clsx';

	type Props = {
		category: ColorCategory;
		default: DefaultColor;
		children: Snippet;
	};

	let { category, default: defaultValue, children }: Props = $props();

	let value = $state(getColor(category));

	const selectOptions = ['custom', ...Object.keys(defaultColors)];

	function set(color: Color) {
		value = color;
		setColor(category, color);
	}
</script>

<div class="flex items-center">
	<Label>{capitalize(category)} color</Label>

	<Info>
		{@render children()}
	</Info>

	<Select
		type="single"
		triggerClass="grow"
		bind:value={
			() => (value.type === 'custom' ? 'custom' : value.name),
			(selectValue) =>
				set(
					selectValue === 'custom'
						? { type: 'custom', hex: '#6b7280' }
						: { type: 'default', name: selectValue }
				)
		}
		items={selectItems(selectOptions, capitalize)}
	>
		{#snippet label({ defaultLabel })}
			{@render colorIcon(value)}

			<div class="text-primary-300">
				{defaultLabel}
			</div>
		{/snippet}

		{#snippet item({ label, value })}
			{@render colorIcon(
				value === 'custom'
					? { type: 'custom', hex: '' }
					: { type: 'default', name: value as DefaultColor },
				'mr-2'
			)}
		{/snippet}
	</Select>

	{#if value.type === 'custom'}
		<input
			type="color"
			class="ml-1 h-full grow"
			bind:value={
				() => (value.type === 'custom' ? value.hex : '#6b7280'),
				(hex) => set({ type: 'custom', hex })
			}
		/>
	{/if}

	<ResetButton class="ml-1" onclick={() => set({ type: 'default', name: defaultValue })} />
</div>

{#snippet colorIcon(value: Color, className?: ClassValue)}
	{#if value.type === 'custom'}
		<Icon class={clsx(className, 'text-primary-400 size-4')} icon="mdi:edit" />
	{:else}
		<span
			class={[className, 'inline-block size-4 rounded-full']}
			style="background-color: {defaultColors[value.name][600]}"
		></span>
	{/if}
{/snippet}
