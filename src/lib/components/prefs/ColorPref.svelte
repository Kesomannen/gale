<script lang="ts">
	import Label from '$lib/components/ui/Label.svelte';
	import { type Color, ColorSetting } from '$lib/state/theme.svelte';
	import { selectItems } from '$lib/util';
	import Select from '$lib/components/ui/Select.svelte';
	import Icon from '@iconify/svelte';
	import ResetButton from '../ui/ResetButton.svelte';
	import type { ClassValue } from 'clsx';
	import Info from '../ui/Info.svelte';
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { DEFAULT_COLORS, type DefaultColor } from '$lib/default-colors';

	type Props = {
		label: string;
		setting: ColorSetting;
		initialCustomColor: string;
		children: Snippet;
	};

	let { label, setting, initialCustomColor, children }: Props = $props();

	const selectOptions = $derived(
		selectItems(
			['custom', ...(setting.hasSystemColor ? ['system'] : []), ...Object.keys(DEFAULT_COLORS)],
			(item) => colorNames[item as keyof typeof colorNames]()
		)
	);

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
		zinc: m.colorPref_color_zinc,
		system: m.colorPref_color_system
	};

	function set(color: Color) {
		setting.current = color;
	}

	function switchColorType<T>(
		value: Color,
		default_: (color: DefaultColor) => T,
		custom: (hex: string) => T,
		system: () => T
	): T {
		switch (value.type) {
			case 'default':
				return default_(value.name);
			case 'custom':
				return custom(value.hex);
			case 'system':
				return system();
		}
	}
</script>

<div class="flex items-center">
	<Label>{label}</Label>

	<Info>
		{@render children()}
	</Info>

	<Select
		type="single"
		triggerClass="grow"
		items={selectOptions}
		bind:value={
			() =>
				switchColorType(
					setting.current,
					(name) => name,
					(_) => 'custom',
					() => 'system'
				),
			(selectValue) =>
				set(
					selectValue === 'custom'
						? { type: 'custom', hex: '#6b7280' }
						: selectValue === 'system'
							? { type: 'system' }
							: { type: 'default', name: selectValue as DefaultColor }
				)
		}
	>
		{#snippet label({ defaultLabel })}
			{@render colorIcon(setting.current)}

			<div class="text-primary-300">
				{defaultLabel}
			</div>
		{/snippet}

		{#snippet item({ value })}
			{@render colorIcon(
				value === 'custom'
					? { type: 'custom', hex: '' }
					: value === 'system'
						? { type: 'system' }
						: { type: 'default', name: value as DefaultColor },
				'mr-2'
			)}
		{/snippet}
	</Select>

	{#if setting.current.type === 'custom'}
		<input
			type="color"
			class="ml-1 h-full grow"
			bind:value={
				() => (setting.current.type === 'custom' ? setting.current.hex : initialCustomColor),
				(hex) => set({ type: 'custom', hex })
			}
		/>
	{/if}

	<ResetButton class="ml-1" onclick={() => set(setting.defaultColor)} />
</div>

{#snippet colorIcon(value: Color, className?: ClassValue)}
	{#if value.type === 'custom'}
		<Icon class={[className, 'text-primary-400 size-4']} icon="mdi:edit" />
	{:else if value.type === 'system'}
		{#await setting.systemColorPromise then color}
			{#if color}
				{@render colorCircle(color, className)}
			{/if}
			<Icon class={[className, 'text-primary-400 size-4']} icon="mdi:monitor" />
		{/await}
	{:else}
		{@render colorCircle(DEFAULT_COLORS[value.name][600], className)}
	{/if}
{/snippet}

{#snippet colorCircle(value: string, className?: ClassValue)}
	<span class={[className, 'inline-block size-4 rounded-full']} style="background-color: {value}"
	></span>
{/snippet}
