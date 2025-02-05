import { useContext, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Link } from 'react-router-dom'
import { CurrentUserContext } from '@/pages/Layout'
import { Spinner } from '@/components/ui/spinner'
import { getAuthLogoutUrl } from '@/lib/useAuth'

function UserProfile() {
	const userState = useContext(CurrentUserContext)

	if (!userState?.reactiveUser) {
		return (
			<Button>Log In</Button>
		)
	}

	const name = userState.attributes?.filter((attribute) => attribute.getName() === "name")?.[0]?.getValue?.() ?? "No Name"

	return (
		<div>
			{name}
		</div>
	)
}

export function SignOutButton() {
	const userState = useContext(CurrentUserContext)
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

export default function AppHeader() {
	const userState = useContext(CurrentUserContext)

	return (
		<div className="bg-lime-500 flex items-center px-8 py-4">
			<Link to="/dashboard">
				<div className='text-3xl'>
					Timecard PRO
				</div>
			</Link>
			<div className="grow">
			</div>
			<div>
				<UserProfile />
			</div>
			{userState?.reactiveUser !== null && (
				<div className='ml-4'>
					<SignOutButton />
				</div>
			)}
		</div>
	)
}