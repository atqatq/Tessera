# SPDX-FileCopyrightText: 2026 The Tessera Project
# SPDX-License-Identifier: Apache-2.0

"""Field dependency graph.

Two jobs the hub cannot do without:

1. Reject cycles at field-definition time, and say exactly which fields
   form the loop. A cycle detected at runtime is an outage.
2. Estimate recompute fan-out before a field is materialised, so a single
   innocent field on a widely referenced object cannot quietly schedule
   millions of rows of work.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, Iterable, List, Sequence, Set


class CycleError(Exception):
    def __init__(self, path: Sequence[str]):
        self.path = list(path)
        super().__init__("dependency cycle: " + " -> ".join(self.path))


@dataclass(frozen=True)
class FanOut:
    field: str
    dependents: List[str]
    estimated_rows: int
    within_budget: bool


class DependencyGraph:
    """Edges point from a field to the fields it reads."""

    def __init__(self):
        self._inputs: Dict[str, Set[str]] = {}

    def add_field(self, name: str, inputs: Iterable[str] = ()) -> None:
        self._inputs.setdefault(name, set()).update(inputs)
        for dep in inputs:
            self._inputs.setdefault(dep, set())

    @property
    def fields(self) -> List[str]:
        return sorted(self._inputs)

    def inputs_of(self, name: str) -> Set[str]:
        return set(self._inputs.get(name, set()))

    def dependents_of(self, name: str) -> Set[str]:
        """Direct dependents: fields that read `name`."""
        return {field for field, deps in self._inputs.items() if name in deps}

    def transitive_dependents(self, name: str) -> List[str]:
        """Every field that must recompute when `name` changes."""
        seen: Set[str] = set()
        frontier = [name]
        while frontier:
            current = frontier.pop()
            for dependent in self.dependents_of(current):
                if dependent not in seen:
                    seen.add(dependent)
                    frontier.append(dependent)
        return sorted(seen)

    def detect_cycle(self) -> None:
        """Raise CycleError with the offending path, or return silently."""
        WHITE, GREY, BLACK = 0, 1, 2
        colour = {field: WHITE for field in self._inputs}
        stack: List[str] = []

        def visit(node: str) -> None:
            colour[node] = GREY
            stack.append(node)
            for dep in sorted(self._inputs.get(node, ())):
                if colour.get(dep, WHITE) == GREY:
                    start = stack.index(dep)
                    raise CycleError(stack[start:] + [dep])
                if colour.get(dep, WHITE) == WHITE:
                    visit(dep)
            stack.pop()
            colour[node] = BLACK

        for field in sorted(self._inputs):
            if colour[field] == WHITE:
                visit(field)

    def topological_order(self) -> List[str]:
        """Evaluation order: every field appears after all of its inputs."""
        self.detect_cycle()
        order: List[str] = []
        seen: Set[str] = set()

        def visit(node: str) -> None:
            if node in seen:
                return
            seen.add(node)
            for dep in sorted(self._inputs.get(node, ())):
                visit(dep)
            order.append(node)

        for field in sorted(self._inputs):
            visit(field)
        return order

    def fan_out(
        self,
        field: str,
        rows_per_field: Dict[str, int],
        budget: int,
        default_rows: int = 0,
    ) -> FanOut:
        """Estimate rows recomputed when `field` changes."""
        dependents = self.transitive_dependents(field)
        estimated = sum(rows_per_field.get(dep, default_rows) for dep in dependents)
        return FanOut(
            field=field,
            dependents=dependents,
            estimated_rows=estimated,
            within_budget=estimated <= budget,
        )


def build_from_definitions(definitions: Iterable[dict]) -> DependencyGraph:
    """Build a graph from field definitions of the form {'field': str, 'inputs': [str]}."""
    graph = DependencyGraph()
    for definition in definitions:
        graph.add_field(definition["field"], definition.get("inputs", []))
    return graph
