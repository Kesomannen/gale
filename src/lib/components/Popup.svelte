<script lang="ts">
	import { Button, Dialog } from 'bits-ui';
	import { fade, scale } from 'svelte/transition';
	import { quartIn, quartOut } from 'svelte/easing';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	export let open: boolean;
	export let title: string | null = null;
	export let confirmClose: { title: string; message: string } | null = null;
	export let canClose: boolean = true;
	export let maxWidth: string | null = null;
	export let onClose: () => void = () => {};

	async function close(evt: UIEvent) {
		if (!canClose) {
			evt.preventDefault();
			return;
		}

		if (confirmClose) {
			evt.preventDefault();
			let result = await confirm(confirmClose.message, { title: confirmClose.title });
			if (!result) {
				return;
			}
		}

		open = false;
		onClose();
	}
</script>

<Dialog.Root
	bind:open
	closeOnEscape={canClose && confirmClose === null}
	closeOnOutsideClick={canClose && confirmClose === null}
	onOutsideClick={close}
>
	<Dialog.Portal>
		<Dialog.Overlay
			data-tauri-drag-region
			class="fixed inset-0 z-0 bg-black/60 rounded-lg"
			transition={fade}
			transitionConfig={{ duration: 150 }}
		/>
		<Dialog.Content
			class="fixed inset-0 flex items-center justify-center pointer-events-none"
			inTransition={scale}
			inTransitionConfig={{ duration: 200, easing: quartOut, start: 0.9 }}
			outTransition={scale}
			outTransitionConfig={{ duration: 100, easing: quartIn, start: 0.95 }}
		>
			<div
				class="z-50 bg-gray-800 rounded-xl p-6 shadow-xl border border-gray-600 overflow-y-auto overflow-x-hidden max-h-[90%] pointer-events-auto
							min-w-[40rem] xl:min-w-[55rem] w-fit max-w-{maxWidth ?? '[85%]'} relative"
			>
				{#if canClose}
					<Button.Root
						class="absolute top-5 right-5 p-0.5 rounded-md text-slate-400 hover:text-red-100 hover:bg-red-600/80 text-3xl"
						on:click={close}
					>
						<Icon icon="mdi:close" />
					</Button.Root>
				{/if}

				{#if title}
					<Dialog.Title class="w-full text-white font-bold text-2xl pr-10">{title}</Dialog.Title>
				{/if}

				<slot />
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
