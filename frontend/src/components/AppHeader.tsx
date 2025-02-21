import { use, useState } from 'react'
import { Link } from 'react-router-dom'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Spinner } from '@/components/ui/spinner'
import { CurrentUserContext } from '@/pages/Layout'
import { getAttributes, getAuthLogoutUrl } from '@/lib/useAuth'

function UserProfile() {
	const userState = use(CurrentUserContext)

	if (!userState?.reactiveUser || !userState.attributes) {
		return (
			<Button>Log In</Button>
		)
	}

	return (
		<div>
			{getAttributes(userState.attributes, "name")?.getValue()}
		</div>
	)
}

export function SignOutButton() {
	const userState = use(CurrentUserContext)
	const [signingOut, setSigningOut] = useState<boolean>(false)

	const signOut = async () => {
		setSigningOut(true);
		await userState?.signOut?.()

		window.location.href = getAuthLogoutUrl();
	}

	return (
		<Button onClick={signOut}>
			{signingOut ? <>Signing Out <Spinner /></> : <>Log Out</>}
		</Button>
	)
}

const getAvatarInitials = (name: string): string => {
	const words = name.trim().split(/\s+/);

	if (words.length === 1) {
		return words[0].substring(0, 2).toUpperCase();
	} else if (words.length === 2 || words.length === 3) {
		return words.map(word => word[0]).join('').toUpperCase();
	} else {
		return (words[0][0] + words[words.length - 1][0]).toUpperCase();
	}
}

export default function AppHeader() {
	const userState = use(CurrentUserContext)

	return (
		<div className="bg-lime-500 flex items-center px-8 py-4">
			<Link to="/dashboard">
				<div className='text-3xl'>
					Timecard PRO
				</div>
			</Link>
			<div className="grow">
			</div>
			<div className='hidden sm:contents'>
				<div>
					<UserProfile />
				</div>
				{userState?.reactiveUser !== null && (
					<div className='ml-4'>
						<SignOutButton />
					</div>
				)}
			</div>
			<div className='sm:hidden'>
				{!!userState?.reactiveUser && !!userState.attributes && (
					<Avatar>
						<AvatarFallback className='text-slate-950'>
							{getAvatarInitials(getAttributes(userState.attributes, 'name')!.getValue())}
						</AvatarFallback>
					</Avatar>
				)}
			</div>
		</div>
	)
}