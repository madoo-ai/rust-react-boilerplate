import { DeleteAccountCard } from "@/components/settings/delete-account-card";
import { ProfileForm } from "@/components/settings/profile-form";
import { useSession } from "@/lib/auth";

export function ProfileSettingsPage() {
  const session = useSession();

  if (!session) return null;

  return (
    <div className="flex flex-col gap-6">
      <ProfileForm
        user={{
          name: session.user.name,
          email: session.user.email,
          phoneNumber: session.user.phoneNumber,
          jobTitle: session.user.jobTitle,
          department: session.user.department,
        }}
      />
      <DeleteAccountCard
        canDelete={session.canDeleteAccount}
        confirmed={session.passwordConfirmed}
      />
    </div>
  );
}
