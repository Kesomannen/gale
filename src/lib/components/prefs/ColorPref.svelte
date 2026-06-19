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
	import { selectItems } from '$lib/util';
	import Select from '$lib/components/ui/Select.svelte';
	import Icon from '@iconify/svelte';
	import ResetButton from '../ui/ResetButton.svelte';
	import type { ClassValue } from 'clsx';
	import Info from '../ui/Info.svelte';
	import type { Snippet } from 'svelte';
	import clsx from 'clsx';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		category: ColorCategory;
		default: DefaultColor;
		children: Snippet;
	};

	let { category, default: defaultValue, children }: Props = $props();

	let value = $state(getColor(category));

	const selectOptions = ['custom', ...Object.keys(defaultColors)];

	const colorNames = {
		amber: m.colorPref_color_amber,
		blue: m.colorPref_color_blue,
		custom: m.colorPref_color_custom,
		cyan: m.colorPref_color_cyan,
		emerald: m.colorPref_color_emerald,
		fuchsia: m.colorPref_color_fuchsia,
		gray: m.colorPref_color_gray,
		green: m.colorPref_color_green,
		indigo: m.colorPref_color_indigo,
		lime: m.colorPref_color_lime,
		neutral: m.colorPref_color_neutral,
		orange: m.colorPref_color_orange,
		pink: m.colorPref_color_pink,
		purple: m.colorPref_color_purple,
		red: m.colorPref_color_red,
		rose: m.colorPref_color_rose,
		sky: m.colorPref_color_sky,
		slate: m.colorPref_color_slate,
		stone: m.colorPref_color_stone,
		teal: m.colorPref_color_teal,
		violet: m.colorPref_color_violet,
		yellow: m.colorPref_color_yellow,
		zinc: m.colorPref_color_zinc
	};

	function set(color: Color) {
		value = color;
		setColor(category, color);
	}
</script>

<div class="flex items-center">
	<Label>{m[`colorPref_title_${category}`]()}</Label>

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
		items={selectItems(selectOptions, (item) => colorNames[item as keyof typeof colorNames]())}
	>
		{#snippet label({ defaultLabel })}
			{@render colorIcon(value)}

			<div class="text-primary-300">
				{defaultLabel}
			</div>
		{/snippet}

		{#snippet item({ value })}
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
