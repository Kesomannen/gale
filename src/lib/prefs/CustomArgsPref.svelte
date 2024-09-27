<script lang="ts">
	import { T, t } from '$i18n';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import Label from '$lib/components/Label.svelte';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	export let value: string[] | null;
	export let set: (value: string[] | null) => Promise<void>;

	let newArg = '';
</script>

<div class="mt-1 flex items-center">
	<Label text={t("Set Launch Arguments")}>
		<p>{@html t("Set Launch Arguments Description 1")}</p>
		<p>
			{t("Set Launch Arguments Description 2")} <code>--foo value</code>{t("Set Launch Arguments Description 3")}
			<code>--foo</code>
			{t("Set Launch Arguments Description 4")} <code>value</code> {t("Set Launch Arguments Description 5")}
		</p>

	</Label>

	<Checkbox
		value={value !== null}
		onValueChanged={(newValue) => {
			set(newValue ? [] : null);
		}}
	/>
</div>

{#if value !== null}
	<div class="mt-1 flex flex-col gap-1 pl-[30%] text-slate-300">
		{#each value as argument, i}
			<div class="flex gap-1">
				<Button.Root
					class="rounded-lg p-1.5 text-xl text-slate-400 hover:bg-gray-700 hover:text-slate-300"
					on:click={() => {
						if (value === null) return;
						value = value.filter((_, index) => index !== i);
						set(value);
					}}
				>
					<Icon icon="mdi:remove" />
				</Button.Root>
				<InputField
					class="flex-grow"
					value={argument}
					on:change={({ detail }) => {
						if (value === null) return;
						value[i] = detail;
						set(value);
					}}
				/>
			</div>
		{/each}

		<InputField
			placeholder={t("Enter new argument")}
			bind:value={newArg}
			on:change={() => {
				if (newArg.length === 0 || value === null) return;

				value = [...value, newArg];
				newArg = '';
				set(value);
			}}
		/>
	</div>
{/if}

<style lang="postcss">
	code {
		@apply bg-gray-900 px-1 text-sm;
	}
</style>
