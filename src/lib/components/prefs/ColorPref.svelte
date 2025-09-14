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
	import { m } from '$lib/paraglide/messages';

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

	const colorLabels: Record<string, () => string> = {
		custom: m.colorPref_color_custom,
		slate: m.colorPref_color_slate,
		gray: m.colorPref_color_gray,
		zinc: m.colorPref_color_zinc,
		neutral: m.colorPref_color_neutral,
		stone: m.colorPref_color_stone,
		red: m.colorPref_color_red,
		orange: m.colorPref_color_orange,
		amber: m.colorPref_color_amber,
		yellow: m.colorPref_color_yellow,
		lime: m.colorPref_color_lime,
		green: m.colorPref_color_green,
		emerald: m.colorPref_color_emerald,
		teal: m.colorPref_color_teal,
		cyan: m.colorPref_color_cyan,
		sky: m.colorPref_color_sky,
		blue: m.colorPref_color_blue,
		indigo: m.colorPref_color_indigo,
		violet: m.colorPref_color_violet,
		purple: m.colorPref_color_purple,
		fuchsia: m.colorPref_color_fuchsia,
		pink: m.colorPref_color_pink,
		rose: m.colorPref_color_rose
	};
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
		items={selectItems(selectOptions, item => {
			return colorLabels[item] ? colorLabels[item]() : capitalize(item);
		})}
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
