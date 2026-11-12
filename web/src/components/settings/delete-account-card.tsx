"use client";

import { useMutation } from "@tanstack/react-query";
import { Trash02Icon as Trash2 } from "@x1nx3r/altics-react-icons";
import { Link, useNavigate } from "react-router-dom";
import { toast } from "sonner";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { api } from "@/lib/api";
import { useAuth } from "@/lib/auth";
import { errorMessage } from "@/lib/errors";
import { useTranslations } from "@/lib/i18n";

export function DeleteAccountCard({
  canDelete,
  confirmed,
}: {
  canDelete: boolean;
  confirmed: boolean;
}) {
  const t = useTranslations("settings.danger");
  const tCommon = useTranslations("common");
  const tAll = useTranslations();
  const navigate = useNavigate();
  const { signOut } = useAuth();

  const mutation = useMutation({
    mutationFn: () => api.delete("/settings/account"),
    onSuccess: async () => {
      toast.success(t("deleted"));
      try {
        await signOut();
      } catch {
        // The account row is gone; clearing the local session is enough.
      }
      navigate("/login?deleted=1", { replace: true });
    },
    onError: (error) => toast.error(errorMessage(error, tAll)),
  });

  return (
    <Card className="border-destructive/30">
      <CardHeader>
        <CardTitle className="text-destructive">{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {!canDelete ? (
          <p className="text-sm text-muted-foreground">
            {tAll("errors.lastSuperadmin")}
          </p>
        ) : !confirmed ? (
          <Button asChild variant="outline">
            <Link to="/confirm-password?next=/settings/profile">
              {t("confirmPassword")}
            </Link>
          </Button>
        ) : (
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="destructive">
                <Trash2 data-icon="inline-start" />
                {t("delete")}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{t("confirmTitle")}</AlertDialogTitle>
                <AlertDialogDescription>
                  {t("confirmDescription")}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel disabled={mutation.isPending}>
                  {tCommon("cancel")}
                </AlertDialogCancel>
                <AlertDialogAction
                  variant="destructive"
                  disabled={mutation.isPending}
                  onClick={(event) => {
                    event.preventDefault();
                    mutation.mutate();
                  }}
                >
                  {mutation.isPending ? <Spinner /> : null}
                  {t("confirm")}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        )}
      </CardContent>
    </Card>
  );
}
