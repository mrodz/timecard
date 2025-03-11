import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { cn } from "@/lib/utils"
import { Button, buttonVariants } from "@/components/ui/button"
import { useState } from "react"
import { ClockSchema } from "@/lib/api"
import { Trash } from "lucide-react"
import { deleteUserClock } from "@/lib/api/clocks"

export type DeleteClockProps = {
	clock: ClockSchema,
	onClockDeleteStart?: () => void;
	onClockDelete?: (clock: ClockSchema | null) => void;
}

export default function DeleteClock(props: DeleteClockProps) {
	const [open, setOpen] = useState(false)
	const [deleting, setDeleting] = useState(false);

	const onCancel = () => {
		setOpen(false);
	}

	const onDelete = async () => {
		props?.onClockDeleteStart?.()
		setDeleting(true)

		const clock = await deleteUserClock({
			clockUuid: props.clock.uuid,
			userPoolId: props.clock.identity_pool_user_id,
		})

		props?.onClockDelete?.(clock)
		setDeleting(false)
		setOpen(false)
	}

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger className={cn(buttonVariants({ variant: 'outline' }), 'self-center w-full mx-auto text-slate-950 gap-0')}>
				Delete "<span className="truncate">{props.clock.name}</span>" <Trash strokeWidth={2} size={32} className="bg-red-500 text-slate-100 rounded-full box-content ml-1 p-1" />
			</DialogTrigger>
			<DialogContent>
				<DialogHeader>
					<DialogTitle className="text-zinc-950">Delete "<span className="truncate">{props.clock.name}</span>"</DialogTitle>
					<DialogDescription>
						PAUSE! Are you sure you wish to delete this clock?
						<br />
						This is a permanent action.
					</DialogDescription>
				</DialogHeader>

				<div className="flex flex-row gap-2">
					<Button variant="ghost" onClick={onCancel}>Cancel</Button>
					<Button disabled={deleting} variant="destructive" onClick={onDelete}>Delete</Button>
				</div>
			</DialogContent>
		</Dialog>
	)
}