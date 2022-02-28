from multiprocessing.sharedctypes import Value


class Segment:
    def __init__(self, segments=None):
        self.segments = segments or []
    
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
