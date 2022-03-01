from itertools import chain

class Segment:
    def __init__(self, segments=None):
        self.segments = segments or []

    @classmethod
    def from_xy(self, segments=None):
        return Segment(segments)
    
    def _add_segment(self, segment):
        self.segments.extend(segment)
    
    def _pad_to_shape(self):
        segments = []
        for (i, elem) in enumerate(self.segments):
            if i > 2 and i % 2 != 0:
                segments.append(self.segments[i])
                segments.append(self.segments[i - 1])
            segments.append(elem)
        return segments[:-2]

    def as_closed_shape(self):
        if len(self.segments) < 2:
            raise ValueError('Cannot close a shape with less than 2 segments')
        
        if self.segments[:2] != self.segments[-2:]:
            self._add_segment(self.segments[:2])

        return self._pad_to_shape()

    def as_open_shape(self):
        return self._pad_to_shape()

    def as_disjoint_segments(self):
        if len(self.segments) % 4 != 0:
            raise ValueError("segments' length must be divisible by 4")
        return self.segments

    def chain_independent(self, other):
        return SegmentCollection([self, other])

    def chain(self, other):
        shape = [*self.segments, *other.segments]
        return SegmentCollection([Segment(shape)])

    def into_collection(self):
        return SegmentCollection([self])


class SegmentCollection:
    def __init__(self, segment_collection=None):
        self.segment_collection = segment_collection or []

    def chain_independent(self, other):
        return SegmentCollection([*self.segment_collection, other])

    def chain(self, other):
        raise NotImplementedError("simple chain not implemented for SegmentCollection")

    def as_closed_shapes(self):
       segments = []
       nested = [bundle.as_closed_shape() for bundle in self.segment_collection]
       segments.extend(chain(*nested))
       return segments

    def as_open_shapes(self):
       segments = []
       nested = [bundle.as_open_shape() for bundle in self.segment_collection]
       segments.extend(chain(*nested))
       return segments

    def into_plottable(self):
       for segment_bundle in self.segment_collection:
              yield segment_bundle.as_open_shape()
