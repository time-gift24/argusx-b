"use client";

import { useState, useEffect } from "react";
import { Plus, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import {
  listChecklistItems,
  createChecklistItem,
  deleteChecklistItem,
  type ChecklistItem,
  type CreateChecklistItemInput,
} from "@/lib/api/prompt-lab";

export default function ChecklistPage() {
  const [items, setItems] = useState<ChecklistItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [prompt, setPrompt] = useState("");
  const [targetLevel, setTargetLevel] = useState<"step" | "sop">("step");
  const [submitting, setSubmitting] = useState(false);

  const loadItems = () => {
    listChecklistItems({}).then((data) => {
      setItems(data);
      setLoading(false);
    });
  };

  useEffect(() => {
    loadItems();
  }, []);

  const handleCreate = async () => {
    if (!name.trim() || !prompt.trim()) return;
    setSubmitting(true);
    try {
      const input: CreateChecklistItemInput = {
        name: name.trim(),
        prompt: prompt.trim(),
        target_level: targetLevel,
        status: "active",
      };
      const newItem = await createChecklistItem(input);
      setItems([...items, newItem]);
      setOpen(false);
      setName("");
      setPrompt("");
      setTargetLevel("step");
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: number) => {
    await deleteChecklistItem(id);
    setItems(items.filter((i) => i.id !== id));
  };

  if (loading) {
    return <div>Loading...</div>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Checklist Items</h1>
        <Sheet open={open} onOpenChange={setOpen}>
          <SheetTrigger asChild>
            <Button>
              <Plus className="h-4 w-4 mr-2" />
              Add Item
            </Button>
          </SheetTrigger>
          <SheetContent>
            <SheetHeader>
              <SheetTitle>Create Checklist Item</SheetTitle>
            </SheetHeader>
            <div className="space-y-4 mt-4">
              <div>
                <Label htmlFor="name">Name</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g., Check JSON syntax"
                />
              </div>
              <div>
                <Label htmlFor="prompt">Prompt</Label>
                <Textarea
                  id="prompt"
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  placeholder="e.g., Validate that the output is valid JSON"
                  rows={3}
                />
              </div>
              <div>
                <Label htmlFor="targetLevel">Target Level</Label>
                <select
                  id="targetLevel"
                  className="w-full border rounded-md px-3 py-2 bg-background"
                  value={targetLevel}
                  onChange={(e) => setTargetLevel(e.target.value as "step" | "sop")}
                >
                  <option value="step">Step</option>
                  <option value="sop">SOP</option>
                </select>
              </div>
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setOpen(false)}>
                  Cancel
                </Button>
                <Button onClick={handleCreate} disabled={submitting || !name.trim() || !prompt.trim()}>
                  {submitting ? "Creating..." : "Create"}
                </Button>
              </div>
            </div>
          </SheetContent>
        </Sheet>
      </div>

      <div className="grid gap-4">
        {items.map((item) => (
          <Card key={item.id}>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>{item.name}</CardTitle>
              <div className="flex items-center gap-2">
                <Badge variant={item.status === "active" ? "default" : "secondary"}>
                  {item.status}
                </Badge>
                <Badge variant="outline">{item.target_level}</Badge>
              </div>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground line-clamp-2">{item.prompt}</p>
              <div className="flex justify-end gap-2 mt-4">
                <Button variant="outline" size="sm">
                  <Pencil className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm" onClick={() => handleDelete(item.id)}>
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
