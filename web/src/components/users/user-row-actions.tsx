"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  DotsHorizontalIcon as MoreHorizontal,
  Pencil01Icon as Pencil,
  Trash02Icon as Trash2,
  UserCheck01Icon as UserCheck,
} from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { Link } from "react-router-dom";
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
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Spinner } from "@/components/ui/spinner";
import { api } from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { useTranslations } from "@/lib/i18n";
import type { UserRow } from "@/types";

export function UserRowActions({
  user,
  canManage,
  canDelete,
}: {
  user: UserRow;
  canManage: boolean;
  canDelete: boolean;
}) {
  const t = useTranslations("users");
  const tAll = useTranslations();
  const queryClient = useQueryClient();
  const [deleteOpen, setDeleteOpen] = useState(false);

  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ["users"] });
    void queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    void queryClient.invalidateQueries({ queryKey: ["user-meta"] });
  };

  const verify = useMutation({
    mutationFn: () => api.post(`/users/${user.id}/verify`),
    onSuccess: () => {
      toast.success(t("messages.verified"));
      invalidate();
    },
    onError: (error) => toast.error(errorMessage(error, tAll)),
  });

  const remove = useMutation({
    mutationFn: () => api.delete(`/users/${user.id}`),
    onSuccess: () => {
      setDeleteOpen(false);
      toast.success(t("messages.deleted"));
      invalidate();
    },
    onError: (error) => toast.error(errorMessage(error, tAll)),
  });

  if (!canManage) {
    return null;
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={tAll("common.openMenu")}
          >
            <MoreHorizontal />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-44">
          <DropdownMenuGroup>
            <DropdownMenuItem asChild>
              <Link to={`/admin/users/${user.id}/edit`}>
                <Pencil />
                {t("actions.edit")}
              </Link>
            </DropdownMenuItem>
            {user.status === "pending" ? (
              <DropdownMenuItem
                onSelect={() => verify.mutate()}
                disabled={verify.isPending}
              >
                <UserCheck />
                {t("actions.verify")}
              </DropdownMenuItem>
            ) : null}
          </DropdownMenuGroup>
          {canDelete ? (
            <>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                variant="destructive"
                onSelect={() => setDeleteOpen(true)}
              >
                <Trash2 />
                {t("actions.delete")}
              </DropdownMenuItem>
            </>
          ) : null}
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("delete.title")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("delete.description", { name: user.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={remove.isPending}>
              {tAll("common.cancel")}
            </AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              disabled={remove.isPending}
              onClick={(event) => {
                event.preventDefault();
                remove.mutate();
              }}
            >
              {remove.isPending ? <Spinner /> : null}
              {t("delete.confirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
