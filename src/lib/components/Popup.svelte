<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { fade, fly } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	import { expoOut, quadIn } from 'svelte/easing';

	type Props = {
		open: boolean;
		title?: string | null;
		confirmClose?: { message: string } | null;
		canClose?: boolean;
		large?: boolean;
		onclose?: () => void;
		children?: import('svelte').Snippet;
	};

	let {
		open = $bindable(),
		title = null,
		confirmClose = null,
		canClose = true,
		large = false,
		onclose,
		children
	}: Props = $props();

	async function close(evt: UIEvent) {
		if (!canClose) {
			evt.preventDefault();
			return;
		}

		if (confirmClose) {
			evt.preventDefault();
			let result = await confirm(confirmClose.message);
			if (!result) return;
		}

		open = false;
		onclose?.();
	}
</script>

<Dialog.Root
	bind:open
	onOpenChange={(open) => {
		if (!open) onclose?.();
	}}
>
	<Dialog.Portal>
		<Dialog.Overlay data-tauri-drag-region forceMount class="pointer-events-none">
			{#snippet child({ props, open })}
				{#if open}
					<div
						{...props}
						transition:fade={{ duration: 80 }}
						class="fixed inset-0 z-0 rounded-lg bg-black/60"
					></div>
				{/if}
			{/snippet}
		</Dialog.Overlay>
		<Dialog.Content
			forceMount
			interactOutsideBehavior={canClose && confirmClose === null ? 'close' : 'ignore'}
			class="pointer-events-none"
		>
			{#if open}
				<div
					class="pointer-events-none fixed inset-0 flex items-center justify-center"
					in:fly={{ duration: 200, easing: expoOut, y: 8 }}
					out:fly={{ duration: 50, easing: quadIn, y: 5 }}
				>
					<div
						class={[
							large ? 'max-w-[60rem]' : 'max-w-[35rem]',
							'border-primary-600 bg-primary-800 pointer-events-auto relative z-50 max-h-[85%] w-[85%] overflow-x-hidden overflow-y-auto rounded-xl border p-6 shadow-xl'
						]}
					>
						{#if canClose}
							<button
								class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 absolute top-5 right-5 rounded-md p-0.5 text-3xl"
								onclick={close}
							>
								<Icon icon="mdi:close" />
							</button>
						{/if}

						{#if title}
							<Dialog.Title class="w-full pr-10 text-2xl font-bold break-words text-white"
								>{title}</Dialog.Title
							>
						{/if}

						{@render children?.()}
					</div>
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
