"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  CheckCircleIcon as CheckCircle2,
  Shield01Icon as ShieldAlert,
} from "@x1nx3r/altics-react-icons";
import { useState } from "react";
import { toast } from "sonner";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Field, FieldContent, FieldLabel } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { api } from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import { useTranslations } from "@/lib/i18n";

export function RegistrationApprovalCard({ enabled }: { enabled: boolean }) {
  const t = useTranslations("users.approval");
  const tMessages = useTranslations("users.messages");
  const tAll = useTranslations();
  const queryClient = useQueryClient();
  const [value, setValue] = useState(enabled);

  const mutation = useMutation({
    mutationFn: (next: boolean) =>
      api.put("/users/registration-approval", { enabled: next }),
    onSuccess: () => {
      toast.success(tMessages("approvalUpdated"));
      void queryClient.invalidateQueries({ queryKey: ["user-meta"] });
      void queryClient.invalidateQueries({ queryKey: ["users"] });
    },
    onError: (error) => {
      setValue(enabled);
      toast.error(errorMessage(error, tAll));
    },
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {value ? <ShieldAlert /> : <CheckCircle2 />}
          {t("title")}
        </CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <Field orientation="horizontal">
          <Switch
            id="registration-approval"
            checked={value}
            disabled={mutation.isPending}
            onCheckedChange={(checked) => {
              setValue(checked);
              mutation.mutate(checked);
            }}
          />
          <FieldContent>
            <FieldLabel htmlFor="registration-approval">
              {t("toggle")}
            </FieldLabel>
            <p className="text-sm text-muted-foreground">
              {value ? t("enabled") : t("disabled")}
            </p>
          </FieldContent>
        </Field>
      </CardContent>
    </Card>
  );
}
